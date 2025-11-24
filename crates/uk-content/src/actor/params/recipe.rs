use std::{
    collections::{BTreeMap, HashMap},
    sync::OnceLock,
};

use anyhow::Context;
use join_str::jstr;
use roead::aamp::Name;
use roead::{aamp::*, byml::Byml};
use serde::{Deserialize, Serialize};
use uk_util::OptionResultExt;

use crate::{
    Result, UKError,
    actor::{InfoSource, ParameterResource},
    prelude::*,
    util::{DeleteMap, IteratorExt},
};

type RecipeTable = DeleteMap<String64, u8>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecipeKeyKind {
    ItemName,
    ItemNum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecipeKeyMatch {
    kind: RecipeKeyKind,
    index: usize,
}

static RECIPE_KEY_MAP: OnceLock<HashMap<u32, RecipeKeyMatch>> = OnceLock::new();

fn recipe_key_map() -> &'static HashMap<u32, RecipeKeyMatch> {
    RECIPE_KEY_MAP.get_or_init(|| {
        let mut map = HashMap::new();

        fn insert_keys(map: &mut HashMap<u32, RecipeKeyMatch>, width: usize) {
            for idx in 0..=999 {
                let name = Name::from_str(format!("ItemName{idx:0width$}").as_str());
                map.insert(
                    name.hash(),
                    RecipeKeyMatch {
                        kind: RecipeKeyKind::ItemName,
                        index: idx,
                    },
                );
                let num = Name::from_str(format!("ItemNum{idx:0width$}").as_str());
                map.insert(
                    num.hash(),
                    RecipeKeyMatch {
                        kind: RecipeKeyKind::ItemNum,
                        index: idx,
                    },
                );
            }
        }

        insert_keys(&mut map, 2);
        insert_keys(&mut map, 3);
        map
    })
}

fn identify_recipe_key(name: &Name) -> Option<RecipeKeyMatch> {
    recipe_key_map().get(&name.hash()).copied()
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]

pub struct Recipe(pub DeleteMap<String64, RecipeTable>);

fn parse_item_index(key: &str, prefix: &str) -> Option<usize> {
    let suffix = key.strip_prefix(prefix)?;
    let bytes = suffix.as_bytes();
    let mut idx = bytes.len();
    while idx > 0 && bytes[idx - 1].is_ascii_digit() {
        idx -= 1;
    }
    if idx == bytes.len() {
        return None;
    }
    suffix[idx..].parse().ok()
}

fn parse_recipe_count(param: &Parameter) -> Result<u8> {
    let raw: i32 = param.as_int()?;
    u8::try_from(raw).map_err(|_| {
        UKError::OtherD(format!(
            "Recipe item count {raw} exceeds supported range for recipe serialization"
        ))
    })
}

fn parse_recipe_table_from_keys(table: &ParameterObject) -> Result<Option<RecipeTable>> {
    let mut entries: BTreeMap<usize, (Option<String64>, Option<u8>)> = BTreeMap::new();
    for (key, value) in table.iter() {
        let key_string = key.to_string();
        if let Some(index) = parse_item_index(&key_string, "ItemName") {
            entries.entry(index).or_insert_with(|| (None, None)).0 = Some(value.as_safe_string()?);
            continue;
        }
        if let Some(index) = parse_item_index(&key_string, "ItemNum") {
            entries.entry(index).or_insert_with(|| (None, None)).1 =
                Some(parse_recipe_count(value)?);
            continue;
        }
        if let Some(key_match) = identify_recipe_key(key) {
            match key_match.kind {
                RecipeKeyKind::ItemName => {
                    entries
                        .entry(key_match.index)
                        .or_insert_with(|| (None, None))
                        .0 = Some(value.as_safe_string()?);
                }
                RecipeKeyKind::ItemNum => {
                    entries
                        .entry(key_match.index)
                        .or_insert_with(|| (None, None))
                        .1 = Some(parse_recipe_count(value)?);
                }
            }
        }
    }

    if entries.is_empty() {
        return Ok(None);
    }

    let mut table_data = RecipeTable::with_capacity(entries.len());
    for (index, (name, count)) in entries {
        let name = name.ok_or_else(|| {
            UKError::MissingAampKeyD(format!("Recipe missing item name at index {index:03}"))
        })?;
        let count = count.ok_or_else(|| {
            UKError::MissingAampKeyD(format!("Recipe missing item count at index {index:03}"))
        })?;
        table_data.insert(name, count);
    }

    Ok(Some(table_data))
}

impl TryFrom<&ParameterIO> for Recipe {
    type Error = UKError;

    fn try_from(pio: &ParameterIO) -> Result<Self> {
        let header = pio
            .object("Header")
            .ok_or(UKError::MissingAampKey("Recipe missing header", None))?;
        let table_count = header
            .get("TableNum")
            .ok_or(UKError::MissingAampKey(
                "Recipe header missing table count",
                None,
            ))?
            .as_int()?;
        let table_names = (0..table_count)
            .named_enumerate("Table")
            .with_padding::<2>()
            .with_zero_index(false)
            .map(|(index, _)| -> Result<String64> {
                Ok(header
                    .get(&index)
                    .ok_or_else(|| {
                        UKError::MissingAampKeyD(jstr!("Recipe header missing table name {&index}"))
                    })?
                    .as_safe_string()?)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(
            table_names
                .into_iter()
                .map(|name| -> Result<(String64, RecipeTable)> {
                    let table = pio.object(name.as_str()).ok_or_else(|| {
                        UKError::MissingAampKeyD(jstr!("Recipe missing table {&name}"))
                    })?;
                    if let Some(entries) = parse_recipe_table_from_keys(table)? {
                        return Ok((name, entries));
                    }
                    let items_count = table
                        .get("ColumnNum")
                        .ok_or(UKError::MissingAampKey(
                            "Recipe table missing column count",
                            None,
                        ))?
                        .as_int()?;
                    let process = |count| -> Result<_> {
                        (1..=count)
                            .named_enumerate("ItemNum")
                            .with_padding::<2>()
                            .with_zero_index(false)
                            .named_enumerate("ItemName")
                            .with_padding::<2>()
                            .with_zero_index(false)
                            .map(|(name, (num, _))| -> Result<(String64, u8)> {
                                Ok((
                                    table
                                        .get(&name)
                                        .ok_or(UKError::MissingAampKey(
                                            "Recipe missing item name",
                                            None,
                                        ))?
                                        .as_safe_string()?,
                                    table
                                        .get(&num)
                                        .ok_or(UKError::MissingAampKey(
                                            "Recipe missing item count",
                                            None,
                                        ))?
                                        .as_int()?,
                                ))
                            })
                            .collect::<Result<_>>()
                            .or_else(|_| {
                                (1..=count)
                                    .named_enumerate("ItemNum")
                                    .with_padding::<3>()
                                    .with_zero_index(false)
                                    .named_enumerate("ItemName")
                                    .with_padding::<3>()
                                    .with_zero_index(false)
                                    .map(|(name, (num, _))| -> Result<(String64, u8)> {
                                        Ok((
                                            table
                                                .get(&name)
                                                .ok_or(UKError::MissingAampKey(
                                                    "Recipe missing item name",
                                                    None,
                                                ))?
                                                .as_safe_string()?,
                                            table
                                                .get(&num)
                                                .ok_or(UKError::MissingAampKey(
                                                    "Recipe missing item count",
                                                    None,
                                                ))?
                                                .as_int()?,
                                        ))
                                    })
                                    .collect::<Result<_>>()
                            })
                    };
                    Ok((
                        name,
                        process(items_count).or_else(|e| {
                            let items_count = (table.0.len() - 1) / 2;
                            process(items_count).context(e)
                        })?,
                    ))
                })
                .collect::<Result<_>>()?,
        ))
    }
}

impl From<Recipe> for ParameterIO {
    fn from(val: Recipe) -> Self {
        Self::new()
            .with_object(
                "Header",
                [("TableNum".into(), Parameter::I32(val.0.len() as i32))]
                    .into_iter()
                    .chain(
                        val.0
                            .keys()
                            .named_enumerate("Table")
                            .with_padding::<2>()
                            .with_zero_index(false)
                            .map(|(index, n)| (index, Parameter::String64(Box::new(*n)))),
                    )
                    .collect(),
            )
            .with_objects(val.0.into_iter().map(|(name, table)| {
                (
                    name,
                    [("ColumnNum".into(), Parameter::I32(table.len() as i32))]
                        .into_iter()
                        .chain(
                            table
                                .into_iter()
                                .filter(|(_, count)| *count > 0)
                                .named_enumerate("ItemNum")
                                .with_padding::<2>()
                                .with_zero_index(false)
                                .named_enumerate("ItemName")
                                .with_padding::<2>()
                                .with_zero_index(false)
                                .flat_map(|(name_idx, (num_idx, (name, count)))| {
                                    [
                                        (name_idx, Parameter::String64(Box::new(name))),
                                        (num_idx, Parameter::I32(count as i32)),
                                    ]
                                }),
                        )
                        .collect(),
                )
            }))
    }
}

impl Mergeable for Recipe {
    fn diff(&self, other: &Self) -> Self {
        other.clone()
    }

    fn merge(&self, diff: &Self) -> Self {
        diff.clone()
    }
}

impl InfoSource for Recipe {
    fn update_info(&self, info: &mut roead::byml::Map) -> crate::Result<()> {
        if let Some(table) = self.0.get(String64::from("Normal0")) {
            info.insert("normal0StuffNum".into(), Byml::I32(table.len() as i32));
            for (name_idx, (num_idx, (name, num))) in table
                .iter()
                .named_enumerate("normal0ItemNum")
                .with_padding::<2>()
                .with_zero_index(false)
                .named_enumerate("normal0ItemName")
                .with_padding::<2>()
                .with_zero_index(false)
            {
                info.insert(name_idx.into(), Byml::String(name.as_str().into()));
                info.insert(num_idx.into(), Byml::I32(*num as i32));
            }
        }
        Ok(())
    }
}

impl ParameterResource for Recipe {
    fn path(name: &str) -> std::string::String {
        jstr!("Actor/Recipe/{name}.brecipe")
    }
}

impl Resource for Recipe {
    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        (&ParameterIO::from_binary(data.as_ref())?).try_into()
    }

    fn into_binary(self, _endian: Endian) -> Vec<u8> {
        ParameterIO::from(self).to_binary()
    }

    fn path_matches(path: impl AsRef<std::path::Path>) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .contains(&"brecipe")
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use roead::aamp::{Name, Parameter, ParameterIO, ParameterObject};

    use crate::{actor::InfoSource, prelude::*};

    #[test]
    fn serde() {
        let actor = crate::tests::test_base_actorpack("Armor_151_Upper");
        let pio = roead::aamp::ParameterIO::from_binary(
            actor
                .get_data("Actor/Recipe/Armor_151_Upper.brecipe")
                .unwrap(),
        )
        .unwrap();
        let recipe = super::Recipe::try_from(&pio).unwrap();
        let data = roead::aamp::ParameterIO::from(recipe.clone()).to_binary();
        let pio2 = roead::aamp::ParameterIO::from_binary(data).unwrap();
        let recipe2 = super::Recipe::try_from(&pio2).unwrap();
        assert_eq!(recipe, recipe2);
    }

    #[test]
    fn diff() {
        let actor = crate::tests::test_base_actorpack("Armor_151_Upper");
        let pio = roead::aamp::ParameterIO::from_binary(
            actor
                .get_data("Actor/Recipe/Armor_151_Upper.brecipe")
                .unwrap(),
        )
        .unwrap();
        let recipe = super::Recipe::try_from(&pio).unwrap();
        let actor2 = crate::tests::test_mod_actorpack("Armor_151_Upper");
        let pio2 = roead::aamp::ParameterIO::from_binary(
            actor2
                .get_data("Actor/Recipe/Armor_151_Upper.brecipe")
                .unwrap(),
        )
        .unwrap();
        let recipe2 = super::Recipe::try_from(&pio2).unwrap();
        let _diff = recipe.diff(&recipe2);
    }

    #[test]
    fn merge() {
        let actor = crate::tests::test_base_actorpack("Armor_151_Upper");
        let pio = roead::aamp::ParameterIO::from_binary(
            actor
                .get_data("Actor/Recipe/Armor_151_Upper.brecipe")
                .unwrap(),
        )
        .unwrap();
        let actor2 = crate::tests::test_mod_actorpack("Armor_151_Upper");
        let recipe = super::Recipe::try_from(&pio).unwrap();
        let pio2 = roead::aamp::ParameterIO::from_binary(
            actor2
                .get_data("Actor/Recipe/Armor_151_Upper.brecipe")
                .unwrap(),
        )
        .unwrap();
        let recipe2 = super::Recipe::try_from(&pio2).unwrap();
        let diff = recipe.diff(&recipe2);
        let merged = recipe.merge(&diff);
        assert_eq!(recipe2, merged);
    }

    #[test]
    fn info() {
        let actor = crate::tests::test_mod_actorpack("Armor_151_Upper");
        let pio = roead::aamp::ParameterIO::from_binary(
            actor
                .get_data("Actor/Recipe/Armor_151_Upper.brecipe")
                .unwrap(),
        )
        .unwrap();
        let recipe = super::Recipe::try_from(&pio).unwrap();
        let mut info = roead::byml::Map::default();
        recipe.update_info(&mut info).unwrap();
        let table = recipe.0.get(String64::from("Normal0")).unwrap();
        assert_eq!(
            info["normal0StuffNum"].as_i32().unwrap(),
            table.len() as i32
        );
        for (i, (name, num)) in table.iter().enumerate() {
            assert_eq!(
                info[format!("normal0ItemName{:02}", i + 1).as_str()]
                    .as_string()
                    .unwrap(),
                name.as_str()
            );
            assert_eq!(
                info[format!("normal0ItemNum{:02}", i + 1).as_str()]
                    .as_i32()
                    .unwrap(),
                *num as i32
            );
        }
    }

    #[test]
    fn identify() {
        let path = std::path::Path::new(
            "content/Actor/Pack/Armor_151_Upper.sbactorpack//Actor/Recipe/Armor_151_Upper.brecipe",
        );
        assert!(super::Recipe::path_matches(path));
    }

    #[test]
    fn zero_indexed_recipe() {
        let header: ParameterObject = [
            (String64::from("TableNum"), Parameter::I32(1)),
            (
                String64::from("Table01"),
                Parameter::String64(Box::new(String64::from("Normal0"))),
            ),
        ]
        .into_iter()
        .collect();
        let table: ParameterObject = [
            (String64::from("ColumnNum"), Parameter::I32(2)),
            (
                String64::from("ItemName00"),
                Parameter::String64(Box::new(String64::from("FirstItem"))),
            ),
            (String64::from("ItemNum00"), Parameter::I32(3)),
            (
                String64::from("ItemName01"),
                Parameter::String64(Box::new(String64::from("SecondItem"))),
            ),
            (String64::from("ItemNum01"), Parameter::I32(1)),
        ]
        .into_iter()
        .collect();
        let pio = ParameterIO::new()
            .with_object("Header", header)
            .with_object("Normal0", table);
        let recipe = super::Recipe::try_from(&pio).unwrap();
        let table = recipe.0.get(String64::from("Normal0")).unwrap();
        let mut iter = table.iter();
        let (first_name, first_count) = iter.next().unwrap();
        assert_eq!(first_name.as_str(), "FirstItem");
        assert_eq!(*first_count, 3);
        let (second_name, second_count) = iter.next().unwrap();
        assert_eq!(second_name.as_str(), "SecondItem");
        assert_eq!(*second_count, 1);
    }

    #[test]
    fn hashed_recipe_keys() {
        let header: ParameterObject = [
            (String64::from("TableNum"), Parameter::I32(1)),
            (
                String64::from("Table01"),
                Parameter::String64(Box::new(String64::from("Normal0"))),
            ),
        ]
        .into_iter()
        .collect();
        let table: ParameterObject = [
            (Name::from_str("ColumnNum"), Parameter::I32(2)),
            (
                Name::from_str("ItemName02"),
                Parameter::String64(Box::new(String64::from("FirstItem"))),
            ),
            (Name::from_str("ItemNum02"), Parameter::I32(3)),
            (
                Name::from_str("ItemName101"),
                Parameter::String64(Box::new(String64::from("SecondItem"))),
            ),
            (Name::from_str("ItemNum101"), Parameter::I32(1)),
        ]
        .into_iter()
        .collect();
        let pio = ParameterIO::new()
            .with_object("Header", header)
            .with_object("Normal0", table);
        let recipe = super::Recipe::try_from(&pio).unwrap();
        let table = recipe.0.get(String64::from("Normal0")).unwrap();
        assert_eq!(table.len(), 2);
        let mut values: Vec<_> = table
            .iter()
            .map(|(name, count)| (name.as_str().to_owned(), *count))
            .collect();
        values.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            values,
            vec![("FirstItem".into(), 3), ("SecondItem".into(), 1)]
        );
    }
}

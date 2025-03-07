use anyhow::Context;
use itertools::Itertools;
use roead::byml::{map, Byml};
use smartstring::alias::String;

use crate::{prelude::Mergeable, util::{parsers::try_get_vecf, DeleteMap}};

use super::AreaShape;

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct NonAutoGenArea {
    pub enable_auto_flower: Option<bool>,
    pub rotate_y:           Option<f32>,
    pub scale:              DeleteMap<char, f32>,
    pub shape:              Option<AreaShape>,
    pub translate:          DeleteMap<char, f32>,
}

impl NonAutoGenArea {
    pub fn id(&self) -> String {
        roead::aamp::hash_name(
            &format!(
                "{}{}{}{}",
                self.translate.values().map(|v| (v * 100000.0f32).to_string()).join(""),
                self.scale.values().map(|v| (v * 100000.0f32).to_string()).join(""),
                self.rotate_y.unwrap_or_default(),
                self.shape.unwrap_or_default(),
            )
        )
        .to_string()
        .into()
    }
}

impl TryFrom<&Byml> for NonAutoGenArea {
    type Error = anyhow::Error;

    fn try_from(value: &Byml) -> anyhow::Result<Self> {
        let map = value.as_map()
            .context("TargetPosMarker node must be HashMap")?;
        Ok(Self {
            enable_auto_flower: Some(map.get("EnableAutoFlower")
                .context("NonAutoGenArea must have EnableAutoFlower")?
                .as_bool()
                .context("NonAutoGenArea EnableAutoFlower must be Bool")?),
            rotate_y: Some(map.get("RotateY")
                .context("NonAutoGenArea must have RotateY")?
                .as_float()
                .context("NonAutoGenArea RotateY must be Float")?),
            scale: try_get_vecf(map.get("Scale")
                .context("NonAutoGenArea must have Scale")?)
                .context("Invalid NonAutoGenArea Scale")?,
            shape: Some(map.get("Shape")
                .context("NonAutoGenArea must have Shape")?
                .try_into()
                .context("NonAutoGenArea has invalid Shape")?),
            translate: try_get_vecf(map.get("Translate")
                .context("NonAutoGenArea must have Translate")?)
                .context("Invalid NonAutoGenArea Translate")?
        })
    }
}

impl From<NonAutoGenArea> for Byml {
    fn from(val: NonAutoGenArea) -> Self {
        map!(
            "EnableAutoFlower" => val.enable_auto_flower.unwrap().into(),
            "RotateY" => val.rotate_y.unwrap().into(),
            "Scale" => Byml::Map(val.scale
                .iter()
                .map(|(k, v)| (k.to_string().into(), Byml::Float(*v)))
                .collect::<crate::util::HashMap<String, Byml>>()),
            "Shape" => (&val.shape.unwrap()).into(),
            "Translate" => Byml::Map(val.translate
                .iter()
                .map(|(k, v)| (k.to_string().into(), Byml::Float(*v)))
                .collect::<crate::util::HashMap<String, Byml>>()),
        )
    }
}

impl Mergeable for NonAutoGenArea {
    fn diff(&self, other: &Self) -> Self {
        Self {
            enable_auto_flower: other.enable_auto_flower
                .ne(&self.enable_auto_flower)
                .then(|| other.enable_auto_flower)
                .unwrap(),
            rotate_y: other.rotate_y
                .ne(&self.rotate_y)
                .then(|| other.rotate_y)
                .unwrap(),
            scale: self.scale.diff(&other.scale),
            shape: other.shape
                .ne(&self.shape)
                .then(|| other.shape)
                .unwrap(),
            translate: self.translate.diff(&other.translate),
        }
    }

    fn merge(&self, diff: &Self) -> Self {
        Self {
            enable_auto_flower: diff.enable_auto_flower
                .eq(&self.enable_auto_flower)
                .then(|| self.enable_auto_flower)
                .or_else(|| Some(diff.enable_auto_flower))
                .unwrap(),
            rotate_y: diff.rotate_y
                .eq(&self.rotate_y)
                .then(|| self.rotate_y)
                .or_else(|| Some(diff.rotate_y))
                .unwrap(),
            scale: self.scale.merge(&diff.scale),
            shape: diff.shape
                .eq(&self.shape)
                .then(|| self.shape)
                .or_else(|| Some(diff.shape))
                .unwrap(),
            translate: self.translate.merge(&diff.translate),
        }
    }
}

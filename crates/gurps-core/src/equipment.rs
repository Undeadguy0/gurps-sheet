//! Equipment (data only; totals and encumbrance are computed in `gurps-rules`).

use crate::{is_blank, LocalizedText};
use serde::{Deserialize, Serialize};

/// Equipment definition from a plugin library. Weight is per unit in lb, cost per unit in $.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EquipmentDef {
    /// Library id; empty for hand-made items and pre-stage-2 files.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub cost: f64,
    /// Can hold other items (backpack, pouch...).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub container: bool,
    /// Short stats line, e.g. "DB 2" or "DR 4/2*" (free text, not used in maths yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
}

impl EquipmentDef {
    pub fn explanation(&self, locale: &str) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.get(locale))
    }
}

fn one() -> u32 {
    1
}
fn yes() -> bool {
    true
}
fn is_true(b: &bool) -> bool {
    *b
}

/// An item the character owns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    #[serde(flatten)]
    pub def: EquipmentDef,
    #[serde(default = "one")]
    pub quantity: u32,
    /// Carried items count towards encumbrance; stored ones (left at home, on a horse) don't.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub carried: bool,
    /// Items inside a container; they are carried only if the container is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Equipment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// User's explanation (Markdown); skipped in JSON when empty.
    #[serde(default, skip_serializing_if = "is_blank")]
    pub explanation: Option<String>,
}

impl Equipment {
    pub fn new(def: EquipmentDef, quantity: u32) -> Self {
        Self {
            def,
            quantity,
            carried: true,
            contents: Vec::new(),
            source: None,
            explanation: None,
        }
    }

    /// Weight of this stack plus its contents (all of it, carried or not).
    pub fn total_weight(&self) -> f64 {
        self.def.weight * self.quantity as f64
            + self
                .contents
                .iter()
                .map(Equipment::total_weight)
                .sum::<f64>()
    }

    pub fn total_cost(&self) -> f64 {
        self.def.cost * self.quantity as f64
            + self.contents.iter().map(Equipment::total_cost).sum::<f64>()
    }

    /// Weight that counts towards encumbrance.
    pub fn carried_weight(&self) -> f64 {
        if !self.carried {
            return 0.0;
        }
        self.def.weight * self.quantity as f64
            + self
                .contents
                .iter()
                .map(Equipment::carried_weight)
                .sum::<f64>()
    }

    /// Item at an index path (`[2, 0]` = first item inside the third one).
    pub fn at_path<'a>(list: &'a [Equipment], path: &[usize]) -> Option<&'a Equipment> {
        let (first, rest) = path.split_first()?;
        let e = list.get(*first)?;
        if rest.is_empty() {
            Some(e)
        } else {
            Self::at_path(&e.contents, rest)
        }
    }

    pub fn at_path_mut<'a>(list: &'a mut [Equipment], path: &[usize]) -> Option<&'a mut Equipment> {
        let (first, rest) = path.split_first()?;
        let e = list.get_mut(*first)?;
        if rest.is_empty() {
            Some(e)
        } else {
            Self::at_path_mut(&mut e.contents, rest)
        }
    }

    /// The list holding the item at `path` (top level or a container's contents).
    pub fn parent_list_mut<'a>(
        list: &'a mut Vec<Equipment>,
        path: &[usize],
    ) -> Option<&'a mut Vec<Equipment>> {
        match path.split_last()? {
            (_, []) => Some(list),
            (_, parent) => Self::at_path_mut(list, parent).map(|e| &mut e.contents),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_shape_loads_and_totals() {
        let e: Equipment =
            serde_json::from_str(r#"{"name":"Rope","quantity":2,"weight":1.5,"cost":5.0}"#)
                .unwrap();
        assert!(e.carried && e.def.id.is_empty());
        assert_eq!((e.total_weight(), e.total_cost()), (3.0, 10.0));
        let s = serde_json::to_string(&e).unwrap();
        assert!(!s.contains("carried") && !s.contains("contents") && !s.contains("\"id\""));
    }

    #[test]
    fn containers_and_paths() {
        let mut pack = Equipment::new(
            EquipmentDef {
                name: "Pack".into(),
                weight: 10.0,
                container: true,
                ..Default::default()
            },
            1,
        );
        let mut knife = Equipment::new(
            EquipmentDef {
                name: "Knife".into(),
                weight: 0.5,
                ..Default::default()
            },
            2,
        );
        pack.contents.push(knife.clone());
        knife.carried = false;
        pack.contents.push(knife);
        let mut list = vec![pack];
        assert_eq!(list[0].total_weight(), 12.0);
        assert_eq!(list[0].carried_weight(), 11.0);
        assert_eq!(Equipment::at_path(&list, &[0, 1]).unwrap().quantity, 2);
        Equipment::parent_list_mut(&mut list, &[0, 1])
            .unwrap()
            .remove(1);
        assert_eq!(list[0].contents.len(), 1);
        list[0].carried = false;
        assert_eq!(list[0].carried_weight(), 0.0);
        let back: Equipment =
            serde_json::from_str(&serde_json::to_string(&list[0]).unwrap()).unwrap();
        assert_eq!(back, list[0]);
    }
}

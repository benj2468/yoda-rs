use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Delta<T> {
    pub start: Option<String>,
    pub end: Option<T>,
}

impl<T> std::default::Default for Delta<T> {
    fn default() -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl<T> Delta<T> {
    pub fn init(init_value: Option<T>) -> Option<Self> {
        init_value.map(|inner| Delta {
            start: None,
            end: Some(inner),
        })
    }
}

pub trait Del<S>: From<S> + Into<S>
where
    S: serde::Serialize,
{
    fn ty() -> String;
    fn apply(&mut self, del: S);
}

pub trait Store {
    fn ty() -> String;
    fn identifier(&self) -> String;
}

pub fn check_delta<T: Debug, S: Hash + PartialEq + Debug>(
    curr: Option<&S>,
    del: Delta<T>,
) -> Option<T> {
    let Delta { start, end } = del;

    if let Some(curr) = curr {
        let mut hasher = DefaultHasher::new();
        curr.hash(&mut hasher);
        let hash = hasher.finish().to_string();

        if let Some(start) = start {
            if hash == start {
                end
            } else {
                None
            }
        } else {
            end
        }
    } else {
        end
    }
}
pub trait Rollup<T> {
    fn rollup(self) -> T;
}
impl<S, B> Rollup<B> for std::vec::IntoIter<S>
where
    S: serde::Serialize,
    B: Del<S> + Default,
{
    fn rollup(self) -> B {
        let mut res = B::default();
        self.for_each(|del| res.apply(del));
        res
    }
}

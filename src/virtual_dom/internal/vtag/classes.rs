use indexmap::set::IndexSet;

/// A set of classes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Classes {
    pub(crate) set: IndexSet<String>,
}

impl Classes {
    /// Creates empty set of classes.
    pub fn new() -> Self {
        Self {
            set: IndexSet::new(),
        }
    }

    /// Adds a class to a set.
    ///
    /// Prevents duplication of class names.
    pub fn push(&mut self, class: &str) {
        self.set.insert(class.into());
    }

    /// Check the set contains a class.
    pub fn contains(&self, class: &str) -> bool {
        self.set.contains(class)
    }

    /// Adds other classes to this set of classes; returning itself.
    ///
    /// Takes the logical union of both `Classes`.
    pub fn extend<T: Into<Classes>>(mut self, other: T) -> Self {
        self.set.extend(other.into().set.into_iter());
        self
    }
}

impl ToString for Classes {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        for class in &self.set {
            buf.push_str(class);
            buf.push(' ');
        }
        buf.pop();
        buf
    }
}

impl From<&str> for Classes {
    fn from(t: &str) -> Self {
        let set = t.split_whitespace().map(String::from).collect();
        Self { set }
    }
}

impl From<String> for Classes {
    fn from(t: String) -> Self {
        let set = t.split_whitespace().map(String::from).collect();
        Self { set }
    }
}

impl From<&String> for Classes {
    fn from(t: &String) -> Self {
        let set = t.split_whitespace().map(String::from).collect();
        Self { set }
    }
}

impl<T: AsRef<str>> From<Vec<T>> for Classes {
    fn from(t: Vec<T>) -> Self {
        let set = t.iter().map(|x| x.as_ref().to_string()).collect();
        Self { set }
    }
}

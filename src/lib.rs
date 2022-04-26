// Copyright (c) 2022 myl7
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, RwLock};

pub struct Tree {
    root: Arc<RwLock<Box<Node>>>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root: Arc::new(RwLock::new(Box::new(Node::new("", Some(""))))),
        }
    }

    pub fn insert(&mut self, path: impl AsRef<str>, id: impl AsRef<str>) {
        let p = path.as_ref().to_owned();
        let mut components = p.split("/").collect::<Vec<_>>();
        components = format_components(components);

        let mut node = self.root.clone();
        for &c in components.iter() {
            let next_node = match node
                .read()
                .unwrap()
                .children
                .iter()
                .find(|child| child.read().unwrap().name.as_str() == c)
            {
                Some(child) => Some(child.clone()),
                None => None,
            };
            node = next_node.unwrap_or_else(|| {
                let mut node_write = node.write().unwrap();
                node_write
                    .children
                    .push(Arc::new(RwLock::new(Box::new(Node::new(
                        c,
                        None as Option<String>,
                    )))));
                node_write.children.last().unwrap().clone()
            });
        }
        node.write().unwrap().id = Some(id.as_ref().to_owned());
    }

    pub fn get(&self, path: impl AsRef<str>) -> Option<String> {
        let p = path.as_ref().to_owned();
        let mut components = p.split("/").collect::<Vec<_>>();
        components = format_components(components);

        let mut node = self.root.clone();
        for &c in components.iter() {
            let next_node = match node
                .read()
                .unwrap()
                .children
                .iter()
                .find(|child| child.read().unwrap().name.as_str() == c)
            {
                Some(child) => child.clone(),
                None => return None,
            };
            node = next_node;
        }
        let id = node.read().unwrap().id.clone();
        id
    }

    pub fn invalid(&self, path: impl AsRef<str>) {
        let p = path.as_ref().to_owned();
        let mut components = p.split("/").collect::<Vec<_>>();
        components = format_components(components);
        let last_component = match components.last() {
            Some(&c) => c,
            None => return,
        };
        components.pop();

        let mut node = self.root.clone();
        for &c in components.iter() {
            let next_node = match node
                .read()
                .unwrap()
                .children
                .iter()
                .find(|child| child.read().unwrap().name.as_str() == c)
            {
                Some(child) => child.clone(),
                None => return,
            };
            node = next_node
        }
        let mut node_write = node.write().unwrap();
        if let Some(i) = node_write
            .children
            .iter()
            .enumerate()
            .find(|(_, child)| child.read().unwrap().name.as_str() == last_component)
            .map(|(i, _)| i)
        {
            node_write.children.remove(i);
        }
    }
}

fn format_components(mut components: Vec<&str>) -> Vec<&str> {
    if components.first().map(|&c| c == "").unwrap_or(false) {
        components.remove(0);
    }
    if components.last().map(|&c| c == "").unwrap_or(false) {
        components.pop();
    }
    components
}

struct Node {
    name: String,
    id: Option<String>,
    children: Vec<Arc<RwLock<Box<Node>>>>,
}

impl Node {
    fn new(name: impl AsRef<str>, id: Option<impl AsRef<str>>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            id: id.map(|i| i.as_ref().to_owned()),
            children: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut tree = Tree::new();
        tree.insert("/test", "0");
        let root = tree.root.read().unwrap();
        assert_eq!(root.children.len(), 1);
        let new_node = root.children[0].read().unwrap();
        assert_eq!(new_node.name, "test");
        assert_eq!(new_node.id, Some("0".to_owned()));
    }

    #[test]
    fn test_get() {
        let mut tree = Tree::new();
        tree.insert("/test", "0");
        tree.insert("/test/test/test", "1");
        assert_eq!(tree.get("/test"), Some("0".to_owned()));
        assert_eq!(tree.get("/test/test/test"), Some("1".to_owned()));
        assert_eq!(tree.get("/test/test"), None);
        assert_eq!(tree.get("/test/test/test/test"), None);
    }

    #[test]
    fn test_invalid() {
        let mut tree = Tree::new();
        tree.insert("/test", "0");
        tree.insert("/test/test", "1");
        tree.insert("/test/test/test", "2");
        tree.invalid("/test/test/test/test");
        tree.invalid("/test/test");
        assert_eq!(tree.get("/test/test/test"), None);
        assert_eq!(tree.get("/test").as_deref(), Some("0"));
    }
}

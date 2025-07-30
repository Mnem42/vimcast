#![allow(unused)]
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct RadixNode {
    pub children: HashMap<String, RadixNode>,
    pub is_end: bool,
}

impl RadixNode {
    pub fn new() -> Self {
        RadixNode {
            children: HashMap::new(),
            is_end: false,
        }
    }

    pub fn insert(&mut self, word: &str) {
        let word = word.to_lowercase();
        for (key, child) in self.children.clone() {
            let common_prefix_len = common_prefix_len(&key, &word);
            if common_prefix_len > 0 {
                let common_prefix = &key[..common_prefix_len];
                let remaining_key = &key[common_prefix_len..];
                let remaining_word = &word[common_prefix_len..];

                let mut new_child = RadixNode::new();
                if !remaining_key.is_empty() {
                    new_child.children.insert(remaining_key.to_string(), child);
                } else {
                    new_child = child;
                }

                if !remaining_word.is_empty() {
                    new_child.insert(remaining_word);
                } else {
                    new_child.is_end = true;
                }

                self.children.remove(&key);
                self.children.insert(common_prefix.to_string(), new_child);
                return;
            }
        }

        self.children.insert(
            word.to_string(),
            RadixNode {
                children: HashMap::new(),
                is_end: true,
            },
        );
    }

    pub fn search(&self, word: &str) -> bool {
        for (key, child) in &self.children {
            if word.to_lowercase().starts_with(key) {
                let remaining = &word[key.len()..];
                if remaining.is_empty() {
                    return child.is_end;
                } else {
                    return child.search(remaining);
                }
            }
        }
        false
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        for (key, child) in &self.children {
            let common_prefix_len = common_prefix_len(key, &prefix.to_lowercase());
            if common_prefix_len == key.len() && common_prefix_len == prefix.len() {
                return true;
            } else if common_prefix_len == key.len() && common_prefix_len < prefix.len() {
                let remaining = &prefix[common_prefix_len..];
                return child.starts_with(remaining);
            } else if common_prefix_len == prefix.len() {
                return true;
            }
        }
        false
    }

    pub fn collect(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        self.collect_helper(prefix, "", &mut results);
        results
    }

    fn collect_helper(&self, prefix: &str, current_path: &str, results: &mut Vec<String>) {
        for (key, child) in &self.children {
            let common_len = common_prefix_len(key, prefix);
            if common_len == prefix.len() {
                // Found the prefix match, collect everything below
                let new_path = format!("{}{}", current_path, key);
                child.collect_all(&new_path, results);
            } else if common_len == key.len() && common_len < prefix.len() {
                let remaining = &prefix[common_len..];
                let new_path = format!("{}{}", current_path, key);
                child.collect_helper(remaining, &new_path, results);
            }
        }
    }

    fn collect_all(&self, path: &str, results: &mut Vec<String>) {
        if self.is_end {
            results.push(path.to_string());
        }
        for (key, child) in &self.children {
            let new_path = format!("{}{}", path, key);
            child.collect_all(&new_path, results);
        }
    }
}

pub fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(ac, bc)| ac == bc)
        .count()
}

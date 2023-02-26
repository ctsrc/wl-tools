use std::ops::RangeInclusive;

/// The root node of a tree, where the edges are [`char`]s and the nodes are `Option<W>` words
///
/// Regarding the `Option<W>` words in the tree, see in particular the following:
/// - [`Self::is_fully_well_formed`]
pub struct WordCharTreeRootNode<'a, W> {
    edges: &'a [WordCharTreeEdge<'a, W>],
}

impl<W> WordCharTreeRootNode<'_, W> {
    /// Get the max depth of the tree
    ///
    /// Measured in number of lowercase [`char`] edges from the root node
    /// to the deepest node in the tree.
    ///
    /// In a [fully well-formed](`Self::is_fully_well_formed`) word char tree, this depth
    /// corresponds to the length in `char`s of the longest word in the tree.
    ///
    /// [`char`]: prim@char
    pub fn get_max_depth(&self) -> usize {
        self.edges
            .iter()
            .map(|edge| edge.get_max_depth(0))
            .max()
            .unwrap_or(0)
    }
    /// The tree is *fully well-formed* as long as either of the following is true:
    /// - The tree is empty, or
    /// - every leaf node (node without child edges) corresponds to a word `W`.
    ///
    /// Additional notes:
    /// - Non-leaf nodes are allowed to have `word: None`.
    /// - Non-leaf nodes are allowed to have `word: Some(W)`.
    ///
    /// The tree is NOT *fully well-formed* if any of the leaf nodes have `word: None`.
    pub fn is_fully_well_formed(&self) -> bool {
        self.edges
            .iter()
            .map(|edge| edge.is_fully_well_formed())
            .all(|b| b)
    }
    /// The tree is *suitable for iterative char search* for words `W` if the following is true:
    /// - Every non-leaf node has `word: None`.
    ///
    /// Additional notes:
    /// - Leaf nodes are allowed to have `word: None`.
    /// - Leaf nodes are allowed to have `word: Some(W)`.
    ///
    /// In *iterative char search*, words are fed into the search one [`char`] at a time.
    /// Because of this, the search will return a match as soon as the shortest match is found.
    ///
    /// Example:
    /// 1. You have a list of words `[..., arm, army, ..., man, ...]`.
    /// 2. You have the string "army man".
    /// 3. You start an iterative char search on the string
    ///    to find words from the list in your string.
    /// 4. The iterative char search will return a match for the word `arm`.
    ///    You wanted to find the word `army`.
    /// 5. In this case, it was not appropriate to use iterative char search,
    ///    because the wordlist was not suitable for iterative char search.
    pub fn is_suitable_for_iterative_char_search(&self) -> bool {
        self.edges
            .iter()
            .map(|edge| edge.is_suitable_for_iterative_char_search())
            .all(|b| b)
    }
}

struct WordCharTreeEdge<'a, W> {
    char_lowercase: char,
    idx_range: RangeInclusive<usize>,
    child_node: WordCharTreeNode<'a, W>,
}

impl<W> WordCharTreeEdge<'_, W> {
    fn get_max_depth(&self, depth_at_parent_node: usize) -> usize {
        self.child_node.get_max_depth(depth_at_parent_node)
    }
    fn is_fully_well_formed(&self) -> bool {
        self.child_node.is_fully_well_formed()
    }
    fn is_suitable_for_iterative_char_search(&self) -> bool {
        self.child_node.is_suitable_for_iterative_char_search()
    }
}

struct WordCharTreeNode<'a, W> {
    word: Option<W>,
    edges: Option<&'a [WordCharTreeEdge<'a, W>]>,
}

impl<W> WordCharTreeNode<'_, W> {
    fn get_max_depth(&self, depth_at_parent_edge: usize) -> usize {
        let curr_depth = depth_at_parent_edge + 1;
        let Some(edges) = self.edges else { return curr_depth };
        edges
            .iter()
            .map(|edge| edge.get_max_depth(curr_depth))
            .max()
            .unwrap_or(curr_depth)
    }
    fn is_fully_well_formed(&self) -> bool {
        let Some(edges) = self.edges else { return self.word.is_some() };
        edges
            .iter()
            .map(|edge| edge.is_fully_well_formed())
            .all(|b| b)
    }
    fn is_suitable_for_iterative_char_search(&self) -> bool {
        let Some(edges) = self.edges else { return true };
        if self.word.is_some() {
            false
        } else {
            edges
                .iter()
                .map(|edge| edge.is_suitable_for_iterative_char_search())
                .all(|b| b)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    pub enum ExampleWords1 {
        Get,
        Give,
        Go,
    }

    pub enum ExampleWords2 {
        Arm,
        Army,
        Man,
    }

    /// A well-formed example empty wordlist
    /// Suitable for iterative char search (although it would be rather pointless in this case :P)
    pub const EXAMPLE_WORDLIST_EMPTY: WordCharTreeRootNode<()> =
        WordCharTreeRootNode { edges: &[] };

    /// A well-formed example wordlist
    /// Suitable for iterative char search
    pub const EXAMPLE_WORDLIST_1: WordCharTreeRootNode<ExampleWords1> = WordCharTreeRootNode {
        edges: &[WordCharTreeEdge {
            char_lowercase: 'g',
            idx_range: 0..=2,
            child_node: WordCharTreeNode {
                word: None,
                edges: Some(&[
                    WordCharTreeEdge {
                        char_lowercase: 'e',
                        idx_range: 0..=0,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: Some(&[WordCharTreeEdge {
                                char_lowercase: 't',
                                idx_range: 0..=0,
                                child_node: WordCharTreeNode {
                                    word: Some(ExampleWords1::Get),
                                    edges: None,
                                },
                            }]),
                        },
                    },
                    WordCharTreeEdge {
                        char_lowercase: 'i',
                        idx_range: 1..=1,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: Some(&[WordCharTreeEdge {
                                char_lowercase: 'v',
                                idx_range: 1..=1,
                                child_node: WordCharTreeNode {
                                    word: None,
                                    edges: Some(&[WordCharTreeEdge {
                                        char_lowercase: 'e',
                                        idx_range: 1..=1,
                                        child_node: WordCharTreeNode {
                                            word: Some(ExampleWords1::Give),
                                            edges: None,
                                        },
                                    }]),
                                },
                            }]),
                        },
                    },
                    WordCharTreeEdge {
                        char_lowercase: 'o',
                        idx_range: 2..=2,
                        child_node: WordCharTreeNode {
                            word: Some(ExampleWords1::Go),
                            edges: None,
                        },
                    },
                ]),
            },
        }],
    };

    /// A well-formed example wordlist
    /// Not suitable for iterative char search
    pub const EXAMPLE_WORDLIST_2: WordCharTreeRootNode<ExampleWords2> = WordCharTreeRootNode {
        edges: &[
            WordCharTreeEdge {
                char_lowercase: 'a',
                idx_range: 0..=1,
                child_node: WordCharTreeNode {
                    word: None,
                    edges: Some(&[WordCharTreeEdge {
                        char_lowercase: 'r',
                        idx_range: 0..=1,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: Some(&[WordCharTreeEdge {
                                char_lowercase: 'm',
                                idx_range: 0..=1,
                                child_node: WordCharTreeNode {
                                    word: Some(ExampleWords2::Arm),
                                    edges: Some(&[WordCharTreeEdge {
                                        char_lowercase: 'y',
                                        idx_range: 1..=1,
                                        child_node: WordCharTreeNode {
                                            word: Some(ExampleWords2::Army),
                                            edges: None,
                                        },
                                    }]),
                                },
                            }]),
                        },
                    }]),
                },
            },
            WordCharTreeEdge {
                char_lowercase: 'm',
                idx_range: 2..=2,
                child_node: WordCharTreeNode {
                    word: None,
                    edges: Some(&[WordCharTreeEdge {
                        char_lowercase: 'a',
                        idx_range: 2..=2,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: Some(&[WordCharTreeEdge {
                                char_lowercase: 'n',
                                idx_range: 2..=2,
                                child_node: WordCharTreeNode {
                                    word: Some(ExampleWords2::Man),
                                    edges: None,
                                },
                            }]),
                        },
                    }]),
                },
            },
        ],
    };

    #[test_case(EXAMPLE_WORDLIST_EMPTY, 0)]
    #[test_case(EXAMPLE_WORDLIST_1, 4)]
    fn test_positive_max_depth_value<W>(root: WordCharTreeRootNode<W>, expected_value: usize) {
        assert_eq!(root.get_max_depth(), expected_value);
    }

    #[test_case(EXAMPLE_WORDLIST_EMPTY)]
    #[test_case(EXAMPLE_WORDLIST_1)]
    fn test_positive_well_formed<W>(root: WordCharTreeRootNode<W>) {
        assert!(root.is_fully_well_formed());
    }

    #[test_case(EXAMPLE_WORDLIST_EMPTY)]
    #[test_case(EXAMPLE_WORDLIST_1)]
    fn test_positive_suitable_iterative_char_search<W>(root: WordCharTreeRootNode<W>) {
        assert!(root.is_suitable_for_iterative_char_search());
    }

    #[test_case(EXAMPLE_WORDLIST_2)]
    fn test_negative_suitable_iterative_char_search<W>(root: WordCharTreeRootNode<W>) {
        assert!(!root.is_suitable_for_iterative_char_search());
    }
}

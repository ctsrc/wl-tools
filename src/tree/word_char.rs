use std::ops::RangeInclusive;

use crate::Words;

/// An iterator over the words of a [`WordCharTreeRootNode`]
struct Iter<'a, W> {
    root: &'a WordCharTreeRootNode<'a, W>,
    curr_edge: usize,
    curr_node: Option<&'a WordCharTreeNode<'a, W>>,
    curr_node_visitor: Option<WordCharTreeNodeVisitor<'a, W>>,
}

impl<'a, W> Iter<'a, W> {
    fn boxed(root: &'a WordCharTreeRootNode<'a, W>) -> Box<Self> {
        let (curr_node, curr_node_visitor) = if root.edges.is_empty() {
            (None, None)
        } else {
            (
                Some(&root.edges[0].child_node),
                Some(WordCharTreeNodeVisitor::new(&root.edges[0].child_node)),
            )
        };
        Box::new(Self {
            root,
            curr_edge: 0,
            curr_node,
            curr_node_visitor,
        })
    }
}

impl<'a, W> Iterator for Iter<'a, W> {
    type Item = &'a W;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(visitor) = &mut self.curr_node_visitor else { return None };
        match visitor.next() {
            None => {
                // Proceed to next edge unless we've already reached the end of the edges.
                if self.curr_edge < self.root.edges.len() {
                    self.curr_edge += 1;
                }
                // If there are no edges left, we signal end of iteration.
                if self.curr_edge == self.root.edges.len() {
                    self.curr_node = None;
                    self.curr_node_visitor = None;
                    return None;
                }

                let curr_node = &self.root.edges[self.curr_edge].child_node;
                self.curr_node = Some(curr_node);
                let curr_node_visitor = WordCharTreeNodeVisitor::new(curr_node);
                self.curr_node_visitor = Some(curr_node_visitor);
                (self.curr_node_visitor.as_mut().unwrap()).next()
            }
            Some(w) => Some(w),
        }
    }
}

struct WordCharTreeNodeVisitor<'a, W> {
    node: &'a WordCharTreeNode<'a, W>,
    has_visited_own_node: bool,
    has_initialized_children: bool,
    curr_edge: usize,
    curr_node: Option<&'a WordCharTreeNode<'a, W>>,
    curr_node_visitor: Option<Box<WordCharTreeNodeVisitor<'a, W>>>,
}

impl<'a, W> WordCharTreeNodeVisitor<'a, W> {
    fn new(node: &'a WordCharTreeNode<'a, W>) -> Self {
        Self {
            node,
            has_visited_own_node: false,
            has_initialized_children: false,
            curr_edge: 0,
            curr_node: None,
            curr_node_visitor: None,
        }
    }
}

impl<'a, W> Iterator for WordCharTreeNodeVisitor<'a, W> {
    type Item = &'a W;

    fn next(&mut self) -> Option<Self::Item> {
        // Visit own node before visiting children
        if !self.has_visited_own_node {
            self.has_visited_own_node = true;
            if self.node.word.is_some() {
                return self.node.word.as_ref();
            }
        }

        if !self.has_initialized_children {
            self.has_initialized_children = true;
            if self.node.edges.is_empty() {
                self.curr_node = None;
                self.curr_node_visitor = None;
            } else {
                let curr_node = &self.node.edges[0].child_node;
                self.curr_node = Some(curr_node);
                self.curr_node_visitor = Some(Box::new(WordCharTreeNodeVisitor::new(curr_node)));
            }
        }

        let Some(visitor) = &mut self.curr_node_visitor else { return None };
        match visitor.next() {
            None => {
                // Proceed to next edge unless we've already reached the end of the edges.
                if self.curr_edge < self.node.edges.len() {
                    self.curr_edge += 1;
                }
                // If there are no edges left, signal to parent that subtree is done.
                if self.curr_edge == self.node.edges.len() {
                    self.curr_node = None;
                    self.curr_node_visitor = None;
                    return None;
                }

                let curr_node = &self.node.edges[self.curr_edge].child_node;
                self.curr_node = Some(curr_node);
                let curr_node_visitor = WordCharTreeNodeVisitor::new(curr_node);
                self.curr_node_visitor = Some(Box::new(curr_node_visitor));
                (self.curr_node_visitor.as_mut().unwrap()).next()
            }
            Some(w) => Some(w),
        }
    }
}

/// The root node of a tree, where the edges are [`char`]s and the nodes are `Option<W>` words
///
/// Regarding the `Option<W>` words in the tree, see in particular the following:
/// - [`Self::is_fully_well_formed`]
/// - [`Self::is_suitable_for_iterative_char_search`]
/// - [`Self::words`]
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
    /// Returns an iterator over the words `W` of a word char tree
    pub fn words(&self) -> Words<W> {
        Words::new(Iter::boxed(self))
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
    edges: &'a [WordCharTreeEdge<'a, W>],
}

impl<W> WordCharTreeNode<'_, W> {
    fn get_max_depth(&self, depth_at_parent_edge: usize) -> usize {
        let curr_depth = depth_at_parent_edge + 1;
        self.edges
            .iter()
            .map(|edge| edge.get_max_depth(curr_depth))
            .max()
            .unwrap_or(curr_depth)
    }
    fn is_fully_well_formed(&self) -> bool {
        self.edges
            .iter()
            .map(|edge| edge.is_fully_well_formed())
            .all(|b| b)
    }
    fn is_suitable_for_iterative_char_search(&self) -> bool {
        if self.edges.is_empty() {
            true
        } else if self.word.is_some() {
            false
        } else {
            self.edges
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

    #[derive(Debug, PartialEq)]
    pub enum ExampleWords1 {
        Get,
        Give,
        Go,
    }

    #[derive(Debug, PartialEq)]
    pub enum ExampleWords2 {
        Arm,
        Army,
        Man,
    }

    #[derive(Debug, PartialEq)]
    pub enum ExampleWords3 {
        A,
    }

    #[derive(Debug, PartialEq)]
    pub enum ExampleWords4 {
        An,
    }

    #[derive(Debug, PartialEq)]
    pub enum ExampleWords5 {
        Ant,
    }

    #[derive(Debug, PartialEq)]
    pub enum ExampleWords6 {
        A,
        An,
        Ant,
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
                edges: &[
                    WordCharTreeEdge {
                        char_lowercase: 'e',
                        idx_range: 0..=0,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: &[WordCharTreeEdge {
                                char_lowercase: 't',
                                idx_range: 0..=0,
                                child_node: WordCharTreeNode {
                                    word: Some(ExampleWords1::Get),
                                    edges: &[],
                                },
                            }],
                        },
                    },
                    WordCharTreeEdge {
                        char_lowercase: 'i',
                        idx_range: 1..=1,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: &[WordCharTreeEdge {
                                char_lowercase: 'v',
                                idx_range: 1..=1,
                                child_node: WordCharTreeNode {
                                    word: None,
                                    edges: &[WordCharTreeEdge {
                                        char_lowercase: 'e',
                                        idx_range: 1..=1,
                                        child_node: WordCharTreeNode {
                                            word: Some(ExampleWords1::Give),
                                            edges: &[],
                                        },
                                    }],
                                },
                            }],
                        },
                    },
                    WordCharTreeEdge {
                        char_lowercase: 'o',
                        idx_range: 2..=2,
                        child_node: WordCharTreeNode {
                            word: Some(ExampleWords1::Go),
                            edges: &[],
                        },
                    },
                ],
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
                    edges: &[WordCharTreeEdge {
                        char_lowercase: 'r',
                        idx_range: 0..=1,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: &[WordCharTreeEdge {
                                char_lowercase: 'm',
                                idx_range: 0..=1,
                                child_node: WordCharTreeNode {
                                    word: Some(ExampleWords2::Arm),
                                    edges: &[WordCharTreeEdge {
                                        char_lowercase: 'y',
                                        idx_range: 1..=1,
                                        child_node: WordCharTreeNode {
                                            word: Some(ExampleWords2::Army),
                                            edges: &[],
                                        },
                                    }],
                                },
                            }],
                        },
                    }],
                },
            },
            WordCharTreeEdge {
                char_lowercase: 'm',
                idx_range: 2..=2,
                child_node: WordCharTreeNode {
                    word: None,
                    edges: &[WordCharTreeEdge {
                        char_lowercase: 'a',
                        idx_range: 2..=2,
                        child_node: WordCharTreeNode {
                            word: None,
                            edges: &[WordCharTreeEdge {
                                char_lowercase: 'n',
                                idx_range: 2..=2,
                                child_node: WordCharTreeNode {
                                    word: Some(ExampleWords2::Man),
                                    edges: &[],
                                },
                            }],
                        },
                    }],
                },
            },
        ],
    };

    /// A well-formed example wordlist
    /// Suitable for iterative char search
    pub const EXAMPLE_WORDLIST_3: WordCharTreeRootNode<ExampleWords3> = WordCharTreeRootNode {
        edges: &[WordCharTreeEdge {
            char_lowercase: 'a',
            idx_range: 0..=0,
            child_node: WordCharTreeNode {
                word: Some(ExampleWords3::A),
                edges: &[],
            },
        }],
    };

    /// A well-formed example wordlist
    /// Suitable for iterative char search
    pub const EXAMPLE_WORDLIST_4: WordCharTreeRootNode<ExampleWords4> = WordCharTreeRootNode {
        edges: &[WordCharTreeEdge {
            char_lowercase: 'a',
            idx_range: 0..=0,
            child_node: WordCharTreeNode {
                word: None,
                edges: &[WordCharTreeEdge {
                    char_lowercase: 'n',
                    idx_range: 0..=0,
                    child_node: WordCharTreeNode {
                        word: Some(ExampleWords4::An),
                        edges: &[],
                    },
                }],
            },
        }],
    };

    /// A well-formed example wordlist
    /// Suitable for iterative char search
    pub const EXAMPLE_WORDLIST_5: WordCharTreeRootNode<ExampleWords5> = WordCharTreeRootNode {
        edges: &[WordCharTreeEdge {
            char_lowercase: 'a',
            idx_range: 0..=0,
            child_node: WordCharTreeNode {
                word: None,
                edges: &[WordCharTreeEdge {
                    char_lowercase: 'n',
                    idx_range: 0..=0,
                    child_node: WordCharTreeNode {
                        word: None,
                        edges: &[WordCharTreeEdge {
                            char_lowercase: 't',
                            idx_range: 0..=0,
                            child_node: WordCharTreeNode {
                                word: Some(ExampleWords5::Ant),
                                edges: &[],
                            },
                        }],
                    },
                }],
            },
        }],
    };

    /// A well-formed example wordlist
    /// Not suitable for iterative char search
    pub const EXAMPLE_WORDLIST_6: WordCharTreeRootNode<ExampleWords6> = WordCharTreeRootNode {
        edges: &[WordCharTreeEdge {
            char_lowercase: 'a',
            idx_range: 0..=2,
            child_node: WordCharTreeNode {
                word: Some(ExampleWords6::A),
                edges: &[WordCharTreeEdge {
                    char_lowercase: 'n',
                    idx_range: 1..=2,
                    child_node: WordCharTreeNode {
                        word: Some(ExampleWords6::An),
                        edges: &[WordCharTreeEdge {
                            char_lowercase: 't',
                            idx_range: 2..=2,
                            child_node: WordCharTreeNode {
                                word: Some(ExampleWords6::Ant),
                                edges: &[],
                            },
                        }],
                    },
                }],
            },
        }],
    };

    #[test_case(EXAMPLE_WORDLIST_EMPTY, 0)]
    #[test_case(EXAMPLE_WORDLIST_1, 4)]
    #[test_case(EXAMPLE_WORDLIST_2, 4)]
    #[test_case(EXAMPLE_WORDLIST_3, 1)]
    #[test_case(EXAMPLE_WORDLIST_4, 2)]
    #[test_case(EXAMPLE_WORDLIST_5, 3)]
    #[test_case(EXAMPLE_WORDLIST_6, 3)]
    fn test_positive_max_depth_value<W>(root: WordCharTreeRootNode<W>, expected_value: usize) {
        assert_eq!(root.get_max_depth(), expected_value);
    }

    #[test_case(EXAMPLE_WORDLIST_EMPTY)]
    #[test_case(EXAMPLE_WORDLIST_1)]
    #[test_case(EXAMPLE_WORDLIST_2)]
    #[test_case(EXAMPLE_WORDLIST_3)]
    #[test_case(EXAMPLE_WORDLIST_4)]
    #[test_case(EXAMPLE_WORDLIST_5)]
    #[test_case(EXAMPLE_WORDLIST_6)]
    fn test_positive_fully_well_formed<W>(root: WordCharTreeRootNode<W>) {
        assert!(root.is_fully_well_formed());
    }

    #[test_case(EXAMPLE_WORDLIST_EMPTY)]
    #[test_case(EXAMPLE_WORDLIST_1)]
    #[test_case(EXAMPLE_WORDLIST_3)]
    #[test_case(EXAMPLE_WORDLIST_4)]
    #[test_case(EXAMPLE_WORDLIST_5)]
    fn test_positive_suitable_iterative_char_search<W>(root: WordCharTreeRootNode<W>) {
        assert!(root.is_suitable_for_iterative_char_search());
    }

    #[test_case(EXAMPLE_WORDLIST_2)]
    #[test_case(EXAMPLE_WORDLIST_6)]
    fn test_negative_suitable_iterative_char_search<W>(root: WordCharTreeRootNode<W>) {
        assert!(!root.is_suitable_for_iterative_char_search());
    }

    #[test_case(EXAMPLE_WORDLIST_EMPTY, vec![])]
    #[test_case(EXAMPLE_WORDLIST_1, vec![&ExampleWords1::Get, &ExampleWords1::Give, &ExampleWords1::Go])]
    #[test_case(EXAMPLE_WORDLIST_2, vec![&ExampleWords2::Arm, &ExampleWords2::Army, &ExampleWords2::Man])]
    #[test_case(EXAMPLE_WORDLIST_3, vec![&ExampleWords3::A])]
    #[test_case(EXAMPLE_WORDLIST_4, vec![&ExampleWords4::An])]
    #[test_case(EXAMPLE_WORDLIST_5, vec![&ExampleWords5::Ant])]
    #[test_case(EXAMPLE_WORDLIST_6, vec![&ExampleWords6::A, &ExampleWords6::An, &ExampleWords6::Ant])]
    fn test_positive_words<W>(root: WordCharTreeRootNode<W>, expected_words: Vec<&W>)
    where
        W: std::fmt::Debug + std::cmp::PartialEq,
    {
        assert_eq!(root.words().collect::<Vec<_>>(), expected_words);
    }
}

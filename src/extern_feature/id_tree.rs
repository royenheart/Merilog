#[cfg(test)]
mod extern_id_tree_tests {
    use id_tree::InsertBehavior::*;
    use id_tree::*;

    #[test]
    fn test_ancestors() {
        let mut tree: Tree<i32> = Tree::new();
        let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
        let node_1 = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        let node_2 = tree.insert(Node::new(2), UnderNode(&node_1)).unwrap();
        let node_2_b = tree.insert(Node::new(3), UnderNode(&node_1)).unwrap();

        let mut ancestors = tree.ancestors(&node_2).unwrap();

        assert_eq!(ancestors.next().unwrap().data(), &1);
        assert_eq!(ancestors.next().unwrap().data(), &0);
        assert!(ancestors.next().is_none());
    }

    #[test]
    fn test_childerns() {
        let mut tree: Tree<i32> = Tree::new();
        let root_id = tree.insert(Node::new(0), AsRoot).unwrap();
        let node_1 = tree.insert(Node::new(1), UnderNode(&root_id)).unwrap();
        let node_2 = tree.insert(Node::new(2), UnderNode(&node_1)).unwrap();
        let node_2_b = tree.insert(Node::new(3), UnderNode(&node_1)).unwrap();

        let mut childerns = tree.children(&root_id).unwrap();

        assert_eq!(childerns.next().unwrap().data(), &1);
        assert!(childerns.next().is_none());
    }
}

//! 符号表

use std::collections::{HashSet};

use id_tree::{Tree, Node, NodeId};

use crate::lex::Tokens;

pub type EnvTree = Tree<Env>;

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct Mident {
    toks: String,
    mtp: Option<Mtype>,
    off: Option<u32>,
    value: Option<String>
}

impl Mident {
    pub fn new(toks: String) -> Self {
        Self {
            toks,
            mtp: None,
            off: None,
            value: None
        }
    }

    pub fn set_mtp(&mut self, mtp: Mtype) {
        self.mtp = Some(mtp);
    }

    pub fn set_off(&mut self, off: u32) {
        self.off = Some(off);
    }

    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }

    pub fn get_mtp(&self) -> Option<&Mtype> {
        self.mtp.as_ref()
    }

    pub fn get_off(&self) -> Option<u32> {
        self.off
    }

    pub fn get_value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    pub fn get_toks(&self) -> &String {
        &self.toks
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct Mtype {
    toks: String
}

impl Mtype {
    fn new(toks: String) -> Self {
        Self {
            toks
        }
    }

    pub fn get_toks(&self) -> &String {
        &self.toks
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct Mconst {
    toks: String,
    mtp: Option<Mtype>,
    value: Option<String>
}

impl Mconst {
    fn new(toks: String) -> Self {
        Self {
            toks,
            mtp: None,
            value: None
        }
    }

    pub fn set_mtp(&mut self, mtp: Mtype) {
        self.mtp = Some(mtp);
    }

    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }

    pub fn get_mtp(&self) -> Option<&Mtype> {
        self.mtp.as_ref()
    }

    pub fn get_value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    pub fn get_toks(&self) -> &String {
        &self.toks
    }
}

#[derive(Debug)]
pub struct Env {
    idents: HashSet<Mident>,
    consts: HashSet<Mconst>,
    types: HashSet<Mtype>
}

impl Env {
    fn new() -> Env {
        Env {
            idents: HashSet::new(),
            consts: HashSet::new(),
            types: HashSet::new()
        }
    }

    fn insert_ident(&mut self, ident: Mident) -> bool {
        self.idents.insert(ident)
    }

    fn insert_const(&mut self, mconst: Mconst) -> bool {
        self.consts.insert(mconst)
    }

    fn insert_type(&mut self, mtype: Mtype) -> bool {
        self.types.insert(mtype)
    }
}

pub struct Envs {
    envs: EnvTree
}

impl Envs {
    pub fn new() -> Self {
        let mut t = EnvTree::new();
        // 全局作用域
        let root = Node::new(Env::new());
        t.insert(root, id_tree::InsertBehavior::AsRoot).unwrap();
        Self { 
            envs: t
        }
    }

    pub fn dump(&self) -> String {
        let mut s = String::from("envs: \n");
        self.envs.write_formatted(&mut s).unwrap();
        s
    }

    pub fn root_id(&self) -> NodeId {
        self.envs.root_node_id().unwrap().clone()
    }

    pub fn insert_env(&mut self, parent_id: NodeId) -> NodeId {
        let env = Node::new(Env::new());
        self.envs.insert(env, id_tree::InsertBehavior::UnderNode(&parent_id)).unwrap()
    }
}


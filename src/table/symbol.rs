//! 符号表

use id_tree::{Node, NodeId, Tree};
use std::collections::HashMap;

/// 符号表：符号名（对于值，是引用该值的量名；对于类型，是类型名） -> vec<内情量>，vec 用于隐藏机制
/// 符号得选取源代码中的符号
type SymbolTable<V> = HashMap<String, Vec<V>>;
/// 命名空间
pub struct NameSpace<Ty, V> {
    types: SymbolTable<Ty>,
    values: SymbolTable<V>,
    aliases: HashMap<String, String>,
}
/// 作用域
type SymbolTree<Ty, V> = Tree<NameSpace<Ty, V>>;

impl<Ty, V> NameSpace<Ty, V> {
    pub fn new() -> Self {
        Self {
            types: SymbolTable::new(),
            values: SymbolTable::new(),
            aliases: HashMap::new(),
        }
    }

    fn types_key(&self, symbol: &str) -> Option<String> {
        let ss = symbol.trim();
        let mut v = vec![ss];
        if let Some(aas) = self.aliases.get(ss) {
            v.push(aas);
        }
        for s in v {
            match self.types.contains_key(s) {
                false => (),
                true => {
                    return Some(s.to_string());
                }
            }
        }

        None
    }

    fn types_contains(&self, symbol: &str) -> Option<&Ty> {
        match self.types_key(symbol) {
            Some(s) => self.types.get(s.as_str()).unwrap().last(),
            None => None,
        }
    }

    fn types_contains_mut(&mut self, symbol: &str) -> Option<&mut Ty> {
        match self.types_key(symbol) {
            Some(s) => self.types.get_mut(s.as_str()).unwrap().last_mut(),
            None => None,
        }
    }

    fn types_add_alias(&mut self, symbol: &str, location: &str) {
        let ss = symbol.trim();
        let sl = location.trim();
        self.aliases.insert(ss.to_string(), sl.to_string());
    }

    fn values_key(&self, symbol: &str) -> Option<String> {
        let s = symbol.trim();
        match self.values.contains_key(s) {
            false => None,
            true => Some(s.to_string()),
        }
    }

    fn values_contains(&self, symbol: &str) -> Option<&V> {
        match self.values_key(symbol) {
            Some(s) => self.values.get(s.as_str()).unwrap().last(),
            None => None,
        }
    }

    fn values_contains_mut(&mut self, symbol: &str) -> Option<&mut V> {
        match self.values_key(symbol) {
            Some(s) => self.values.get_mut(s.as_str()).unwrap().last_mut(),
            None => None,
        }
    }

    /// 向类型符号表添加符号，同时贯彻隐藏机制
    fn types_insert(&mut self, key: &str, value: Ty) {
        let k = key.trim();
        if self.types.contains_key(k) {
            self.types.get_mut(k).unwrap().push(value);
        } else {
            self.types.insert(k.to_string(), vec![value]);
        }
    }

    /// 向值符号表添加符号，同时贯彻隐藏机制
    fn values_insert(&mut self, key: &str, value: V) {
        let k = key.trim();
        if self.values.contains_key(k) {
            self.values.get_mut(k).unwrap().push(value);
        } else {
            self.values.insert(k.to_string(), vec![value]);
        }
    }

    fn clear(&mut self) {
        self.types.clear();
        self.values.clear();
    }
}

pub struct SymbolManager<Ty, V> {
    symbols: SymbolTree<Ty, V>,
}

/// 不直接区分类型、变量
/// 存在隐藏机制（inkwell 支持，只是会自动转换 IR 代码中各个指示的名称，包括全局的结构体、字符串、函数、函数内部的变量名称）
/// 但到高级语言层次会有所不同
/// 隐藏机制：（其实就是命名空间的关系）（针对于同一个作用域）
/// 1. 结构体类型声明和函数定义不冲突，引用时需要看具体的方式，都可以被引用到。结构体函数不与函数定义一个作用域下，因此不必担心
/// 2. 结构体类型声明和声明之间，函数定义和函数定义之间，若有重名会冲突
/// 3. 函数体内各个变量的声明不冲突，但是最后的会隐藏前面的
impl<Ty, V> SymbolManager<Ty, V> {
    pub fn new() -> Self {
        let mut tree = SymbolTree::new();
        let root = Node::new(NameSpace::new());
        tree.insert(root, id_tree::InsertBehavior::AsRoot).unwrap();
        SymbolManager { symbols: tree }
    }

    /// 返回符号表
    pub fn symbols(&self) -> &SymbolTree<Ty, V> {
        &self.symbols
    }

    /// 返回符号表当前深度
    pub fn symbols_level(&self) -> usize {
        self.symbols.height()
    }

    /// 顶部作用域
    pub fn root_env(&self) -> NodeId {
        self.symbols.root_node_id().unwrap().clone()
    }

    /// 只查看当前作用域的类型命名空间是否存在该符号，若存在，返回其值
    pub fn looknow_types(&self, symbol: &str, child: &NodeId) -> Option<&Ty> {
        if let Ok(node) = self.symbols.get(child) {
            let t = node.data();
            return t.types_contains(symbol);
        }

        None
    }

    pub fn looknow_types_mut(&mut self, symbol: &str, child: &NodeId) -> Option<&mut Ty> {
        if let Ok(node) = self.symbols.get_mut(child) {
            let t = node.data_mut();
            return t.types_contains_mut(symbol);
        }

        None
    }

    /// 只查看当前作用域的值命名空间是否存在该符号，若存在，返回其值
    pub fn looknow_values(&self, symbol: &str, child: &NodeId) -> Option<&V> {
        if let Ok(node) = self.symbols.get(child) {
            let t = node.data();
            return t.values_contains(symbol);
        }

        None
    }

    pub fn looknow_values_mut(&mut self, symbol: &str, child: &NodeId) -> Option<&mut V> {
        if let Ok(node) = self.symbols.get_mut(child) {
            let t = node.data_mut();
            return t.values_contains_mut(symbol);
        }

        None
    }

    /// 查看当前作用域以及其祖先作用域的类型命名空间是否存在该符号，若存在，返回其值
    pub fn lookup_types(&self, symbol: &str, child: &NodeId) -> Option<&Ty> {
        // let r = self.looknow_types(symbol, child);
        // if r.is_some() {
        //     return r;
        // }
        // let a = self.symbols.ancestors(child).unwrap();

        // for node in a {
        //     let t = node.data();
        //     let tr = t.types_contains(symbol);
        //     if tr.is_some() {
        //         return tr;
        //     }
        // }
        let liter = vec![self.symbols.get(child).unwrap()];
        let riter = self.symbols.ancestors(child).unwrap();
        let iter = liter.into_iter().chain(riter.into_iter());

        for node in iter {
            let t = node.data();
            let tr = t.types_contains(symbol);
            if tr.is_some() {
                return tr;
            }
        }

        None
    }

    /// 查看当前作用域以及其祖先作用域的值命名空间是否存在该符号，若存在，返回其值
    pub fn lookup_values(&self, symbol: &str, child: &NodeId) -> Option<&V> {
        // let r = self.looknow_values(symbol, child);
        // if r.is_some() {
        //     return r;
        // }
        // let a = self.symbols.ancestors(child).unwrap();

        // for node in a {
        //     let t = node.data();
        //     let tr = t.values_contains(symbol);
        //     if tr.is_some() {
        //         return tr;
        //     }
        // }
        let liter = vec![self.symbols.get(child).unwrap()];
        let riter = self.symbols.ancestors(child).unwrap();
        let iter = liter.into_iter().chain(riter.into_iter());

        for node in iter {
            let t = node.data();
            let tr = t.values_contains(symbol);
            if tr.is_some() {
                return tr;
            }
        }

        None
    }

    pub fn types_add_alias(&mut self, symbol: &str, location: &str, root: &NodeId) {
        let tables = self.symbols.get_mut(root).unwrap().data_mut();
        tables.types_add_alias(symbol, location);
    }

    /// 在指定作用域下创建新的子作用域
    pub fn create_env(&mut self, root: &NodeId) -> NodeId {
        let child = Node::new(NameSpace::new());
        self.symbols
            .insert(child, id_tree::InsertBehavior::UnderNode(root))
            .unwrap()
    }

    /// 消除指定作用域中的所有符号
    pub fn clear_env(&mut self, root: &NodeId) {
        let tables = self.symbols.get_mut(root).unwrap().data_mut();
        tables.clear();
    }

    /// 向指定的作用域的类型命名空间添加符号，由命名空间负责隐藏机制
    pub fn push_symbol_types(&mut self, symbol: &str, id: Ty, child: &NodeId) -> Result<(), &Ty> {
        if let Ok(node) = self.symbols.get_mut(child) {
            let t = node.data_mut();
            t.types_insert(symbol, id);
        }

        Ok(())
    }

    /// 向指定的作用域的值命名空间添加符号，由命名空间负责隐藏机制
    pub fn push_symbol_values(&mut self, symbol: &str, id: V, child: &NodeId) -> Result<(), &V> {
        if let Ok(node) = self.symbols.get_mut(child) {
            let t = node.data_mut();
            t.values_insert(symbol, id);
        }

        Ok(())
    }
}

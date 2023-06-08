//! LL1 递归下降分析

use id_tree::{Node, NodeId};
use id_tree::InsertBehavior::AsRoot;
use id_tree::InsertBehavior::UnderNode;
use id_tree::PreOrderTraversal;
use id_tree::PostOrderTraversal;
use id_tree::LevelOrderTraversal;
use crate::lex::Tokens;
use crate::lex::analysis::Analysis;
use crate::mistakes::show::Mis;
use super::{AST, NT};
use super::{ASTNode};

macro_rules! insert_t {
    ($tree: expr, $root: expr, $tok: expr) => {
        $tree.insert(Node::new(ASTNode::T($tok)), UnderNode(&$root)).unwrap()
    };
}

macro_rules! insert_nt {
    ($tree: expr, $root: expr, $type: expr) => {
        $tree.insert(Node::new(ASTNode::NT($type)), UnderNode(&$root)).unwrap()
    };
}

macro_rules! replace {
    ($tree: expr, $root: expr, $type: expr) => {
        $tree.get_mut($root).unwrap().replace_data($type);
    };
}

pub struct RecursiveDescentParser {
    // 词法单元流
    tokens: Vec<Tokens>,
    // 当前分析词法索引
    current: usize,
    tree: AST
}

impl RecursiveDescentParser {
    pub fn new(lexer: Analysis) -> Result<RecursiveDescentParser, Mis> {
        let mut tree = AST::new();
        let root: Node<ASTNode> = Node::new(ASTNode::NT(NT::Merilog));
        tree.insert(root, AsRoot).unwrap();
        let mut tokens: Vec<Tokens> = lexer
            .filter(|x: &Tokens| !matches!(x, Tokens::CommentBlock(_) | Tokens::CommentModule(_)))
            .collect();
        tokens.push(Tokens::End);
        Ok(RecursiveDescentParser { 
            tokens,
            current: 0, tree
        })
    }

    /// 打印
    #[cfg(debug_assertions)]
    pub fn print_test(&self) {
        let mut s = String::new();
        self.tree.write_formatted(&mut s).unwrap();
        println!("{}", s);
    }

    #[cfg(debug_assertions)]
    pub fn get_current(&self) -> &Tokens {
        &self.tokens[self.current]
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        self.tree.write_formatted(&mut s).unwrap();
        s
    }

    /// 前序遍历生成
    pub fn ast_pre_order(&self) -> PreOrderTraversal<ASTNode> {
        let id = &self.root_id();
        self.tree.traverse_pre_order(id).unwrap()
    }

    pub fn ast_post_order(&self) -> PostOrderTraversal<ASTNode> {
        let id = &self.root_id();
        self.tree.traverse_post_order(id).unwrap()
    }

    pub fn ast_level_order(&self) -> LevelOrderTraversal<ASTNode> {
        let id = &self.root_id();
        self.tree.traverse_level_order(id).unwrap()
    }

    pub fn get_ast(&self) -> &AST {
        &self.tree
    }

    pub fn get_tokens(&self) -> &Vec<Tokens> {
        &self.tokens
    }

    /// 返回 AST 头节点索引
    pub fn root_id(&self) -> NodeId {
        self.tree.root_node_id().unwrap().clone()
    }

    pub fn parse(&mut self) -> bool {
        let id = &self.root_id();
        self.match_merilog(id)
    }

    /// 返回当前匹配词法单元的上一个词法单元
    fn copy_pre(&self) -> Option<Tokens> {
        if self.current == 0 {return None;}
        Some(self.tokens[self.current - 1].clone())
    }

    /// 返回当前匹配词法单元
    fn copy_now(&self) -> Option<Tokens> {
        if self.current >= self.tokens.len() {return None;}
        Some(self.tokens[self.current].clone())
    }
    
    /// 将顺序子节点合并为一个节点，进行化简
    fn adjust_single_child(&mut self, node: NodeId) {
        let c_num = self.tree.children(&node).unwrap().count();

        if c_num != 1 {
            return;
        }
        self.tree.remove_node(node, id_tree::RemoveBehavior::LiftChildren).unwrap();
    }

    fn term_str(&mut self, next: bool, insert: Option<&NodeId>) -> Option<Tokens> {
        if self.current >= self.tokens.len() {
            return None;
        }

        if let Tokens::Str(_) = self.tokens[self.current] {
            match next {
                true => self.current += 1,
                false => ()
            };
            let r = self.copy_pre();
            if let (Some(id), Some(tok)) = (insert, r) {
                insert_t!(self.tree, id, tok);
            }
            return self.copy_pre();
        }

        None
    }

    fn term_int(&mut self, next: bool, insert: Option<&NodeId>) -> Option<Tokens> {
        if self.current >= self.tokens.len() {
            return None;
        }
        
        if let Tokens::Int(_) = self.tokens[self.current] {
            match next {
                true => self.current += 1,
                false => ()
            };
            let r = self.copy_pre();
            if let (Some(id), Some(tok)) = (insert, r) {
                insert_t!(self.tree, id, tok);
            }
            return self.copy_pre();
        }

        None
    }

    fn term_decimal(&mut self, next: bool, insert: Option<&NodeId>) -> Option<Tokens> {
        if self.current >= self.tokens.len() {
            return None;
        }

        if let Tokens::Decimal(_) = self.tokens[self.current] {
            match next {
                true => self.current += 1,
                false => ()
            };
            let r = self.copy_pre();
            if let (Some(id), Some(tok)) = (insert, r) {
                insert_t!(self.tree, id, tok);
            }
            return self.copy_pre();
        }

        None
    }

    fn term_bool(&mut self, next: bool, insert: Option<&NodeId>) -> Option<Tokens> {
        if self.current >= self.tokens.len() {
            return None;
        }

        if let Tokens::Bool(_) = self.tokens[self.current] {
            match next {
                true => self.current += 1,
                false => ()
            };
            let r = self.copy_pre();
            if let (Some(id), Some(tok)) = (insert, r) {
                insert_t!(self.tree, id, tok);
            }
            return self.copy_pre();
        }

        None
    }

    fn term_identity(&mut self, next: bool, insert: Option<&NodeId>) -> Option<Tokens> {
        if self.current >= self.tokens.len() {
            return None;
        }

        if let Tokens::Identity(_) = self.tokens[self.current] {
            match next {
                true => self.current += 1,
                false => ()
            };
            let r = self.copy_pre();
            if let (Some(id), Some(tok)) = (insert, r) {
                insert_t!(self.tree, id, tok);
            }
            return self.copy_pre();
        }

        None
    }

    /// 匹配当前 token 是否与需要的 token 相匹配
    fn term(&mut self, tok: Tokens, next: bool, insert: Option<&NodeId>) -> bool {
        // 当前分析索引大于等于词法单元流长度，返回 false
        if self.current >= self.tokens.len() {
            return false;
        }

        // 当前索引指示的词法单元等于判断的词法单元，返回 true
        if self.tokens[self.current] == tok {
            match next {
                true => self.current += 1,
                false => ()
            };
            if let Some(id) = insert {
                insert_t!(self.tree, id, tok);
            }
            return true;
        }

        false
    }

    /// 匹配当前 token 是否属于给定的 toks 
    fn terms(&mut self, toks: Vec<Tokens>, next: bool, insert: Option<&NodeId>) -> bool {
        for tok in toks {
            match self.term(tok, next, insert) {
                false => (),
                true => return true
            }
        }
        false
    }

    fn match_merilog(&mut self, root: &NodeId) -> bool {
        loop {            
            match self.tokens[self.current] {
                Tokens::Struct => {
                    if !self.match_define_struct(root) {
                        return false;
                    }
                },
                Tokens::Fn => {
                    if !self.match_define_fn(root) {
                        return false;
                    }
                },
                Tokens::End => {
                    return true;
                },
                _ => {
                    println!("程序应包含结构体定义或函数，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
                    return false;
                }
            }
        }
    }

    fn match_define_struct(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineStruct);

        if self.term(Tokens::Struct, true, None) && 
            self.term_identity(true, Some(&me)).is_some() && 
            self.term(Tokens::LeftBC, true, None) && 
            self.match_define_struct_body(&me) && 
            self.term(Tokens::RightBC, true, None) {
            return true;
        };

        println!("结构体内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_struct_body(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineStructBody);

        'l: {
            if self.term_identity(true, Some(&me)).is_some() {
                if self.term(Tokens::Semicolon, true, None) && 
                    self.match_exec_type(&me) &&
                    self.match_define_struct_body_next(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::Fn, false, None) {
                if self.match_define_fn(&me) && 
                    self.match_define_struct_body_next(&me) {
                    return true;
                }
                break 'l;
            }
        }

        println!("结构体内域定义问题，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_struct_body_next(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineStructBodyNext);
        
        'l: {
            if self.term(Tokens::Comma, true, None) {
                if self.match_define_struct_body(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightBC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }
    
        println!("结构体域分割错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_var(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineVar);

        if self.term(Tokens::Let, true, None) &&
            self.match_define_var_mutable(&me) {
            return true;
        }

        println!("声明语句内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_var_mutable(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineVarMutable);

        'l: {
            if self.term(Tokens::Mut, true, Some(&me)) {
                if self.match_define_var_s(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term_identity(false, None).is_some() {
                if self.match_define_var_s(&me) {
                    return true;
                }
                break 'l;
            }
        }

        println!("声明可变错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_var_s(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineVarS);

        if self.term_identity(true, Some(&me)).is_some() && 
            self.match_define_var_type(&me) &&
            self.match_define_var_value(&me) && 
            self.match_define_var_e(&me) {
            return true;
        }

        println!("声明单元错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_var_type(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineVarType);

        'l: {
            if self.term(Tokens::Semicolon, true, None) {
                if self.match_exec_type(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![Tokens::Is, Tokens::Comma, Tokens::EndExp], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("声明类型错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }
    
    fn match_define_var_value(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineVarValue);

        'l: {
            if self.term(Tokens::Is, true, None) {
                if self.match_exec_exp(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![Tokens::Comma, Tokens::EndExp], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("声明赋值错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }   

    fn match_define_var_e(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineVarE);

        'l: {
            if self.term(Tokens::Comma, true, None) {
                if self.match_define_var_s(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::EndExp, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("声明分割错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    } 

    fn match_exec_sentence(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecSentence);

        'l: {
            if self.term(Tokens::Let, false, None) {
                if self.match_exec_stmt(&me) &&
                    self.term(Tokens::EndExp, true, None) {
                    return true;
                }
                break 'l;
            }

            if self.term_identity(false, None).is_some() {
                if self.match_exec_is(&me) &&
                    self.term(Tokens::EndExp, true, None) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::If, false, None) {
                if self.match_exec_if(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::While, false, None) {
                if self.match_exec_while(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::LeftBC, false, None) {
                if self.match_exec_loop(&me) &&
                    self.term(Tokens::EndExp, true, None) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::Return, false, None) {
                if self.match_exec_ret(&me) &&
                    self.term(Tokens::EndExp, true, None) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::Break, false, None) {
                if self.match_exec_break(&me) &&
                    self.term(Tokens::EndExp, true, None) {
                    return true;
                }
                break 'l;
            }
        }

        println!("语句内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp(&mut self, root: &NodeId) -> bool {
        let cur: usize = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExp);

        'l: {
            if self.match_exec_exp_ands(&me) {
                if self.match_exec_r1(&me) {
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r1(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR1);

        'l: {
            if self.term(Tokens::OrS, true, Some(&me)) {
                if self.match_exec_exp(&me) &&
                    self.match_exec_r1(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS, 
                Tokens::Comma, Tokens::EndExp, 
                Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("OrS 语句分割错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_ands(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpAndS);

        'l: {
            if self.match_exec_exp_or(&me) {
                if self.match_exec_r2(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }
    
    fn match_exec_r2(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR2);

        'l: {
            if self.term(Tokens::AndS, true, Some(&me)) {
                if self.match_exec_exp_ands(&me) &&
                    self.match_exec_r2(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Comma, 
                Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("AndS 表达式分割错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_or(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpOr);

        'l: {
            if self.match_exec_exp_and(&me) {
                if self.match_exec_r3(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r3(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR3);

        'l: {
            if self.term(Tokens::Or, true, Some(&me)) {
                if self.match_exec_exp_or(&me) &&
                    self.match_exec_r3(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Or, 
                Tokens::Comma, Tokens::EndExp, 
                Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("Or 表达式错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_and(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpAnd);

        'l: {
            if self.match_exec_exp_eq(&me) {
                if self.match_exec_r4(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r4(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR4);

        'l: {
            if self.term(Tokens::And, true, Some(&me)) {
                if self.match_exec_exp_and(&me) &&
                    self.match_exec_r4(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Or, Tokens::And, 
                Tokens::Comma, Tokens::EndExp, 
                Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("And 表达式问题，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_eq(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpEq);

        'l: {
            if self.match_exec_exp_lgq(&me) {
                if self.match_exec_r5(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r5(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR5);

        'l: {
            if self.match_eqs(&me) {
                if self.match_exec_exp_eq(&me) &&
                    self.match_exec_r5(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Or, Tokens::And,
                Tokens::Eq, Tokens::Ne, 
                Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("等于非等于表达式错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_lgq(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpLGq);

        'l: {
            if self.match_exec_exp_add_op(&me) {
                if self.match_exec_r6(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r6(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR6);

        'l: {
            if self.match_lgqs(&me) {
                if self.match_exec_exp_lgq(&me) &&
                    self.match_exec_r6(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Or, Tokens::And,
                Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt,
                Tokens::Ge, Tokens::Le, Tokens::Comma, 
                Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }
        
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_add_op(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpAddOp);

        'l: {
            if self.match_exec_exp_multi_op(&me) {
                if self.match_exec_r7(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r7(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR7);

        'l: {
            if self.match_add_ops(&me) {
                if self.match_exec_exp_add_op(&me) &&
                    self.match_exec_r7(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Or, Tokens::And,
                Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt,
                Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, 
                Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_multi_op(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpMultiOp);

        'l: {
            if self.match_exec_exp_sig_op(&me) {
                if self.match_exec_r8(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_r8(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecR8);

        'l: {
            if self.match_multi_ops(&me) {
                if self.match_exec_exp_multi_op(&me) &&
                    self.match_exec_r8(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::RightC, Tokens::OrS,
                Tokens::AndS, Tokens::Or, Tokens::And,
                Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt,
                Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus,
                Tokens::Mul, Tokens::Div, Tokens::Mod, 
                Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_sig_op(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpSigOp);

        'l: {
            if self.match_sig_ops(&me) {
                if self.match_exec_exp_n(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l; 
            }
        }

        println!("表达式内错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_exp_n(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecExpN);

        'l: {
            if self.term(Tokens::LeftC, true, Some(&me)) {
                if self.match_exec_exp(&me) &&
                    self.term(Tokens::RightC, true, Some(&me)) {
                    return true;       
                }
                break 'l;
            }

            if self.term_str(false, None).is_some() ||
                self.term_int(false, None).is_some() || 
                self.term_decimal(false, None).is_some() || 
                self.term_bool(false, None).is_some() || 
                self.term_identity(false, None).is_some() ||
                self.term(Tokens::Match, false, None) {
                if self.match_ops(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l;
            }
        }

        println!("括号或单元成员错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_eqs(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::Eqs);

        if self.terms(vec![
            Tokens::Eq, Tokens::Ne
        ], true, Some(&me)) {
            self.adjust_single_child(me);
            return true;
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_lgqs(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::LGqs);

        if self.terms(vec![
            Tokens::Gt, Tokens::Lt,
            Tokens::Ge, Tokens::Le
        ], true, Some(&me)) {
            self.adjust_single_child(me);
            return true;
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_add_ops(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::AddOps);

        if self.terms(vec![
            Tokens::Plus, Tokens::Minus
        ], true, Some(&me)) {
            self.adjust_single_child(me);
            return true;
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_multi_ops(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::MultiOps);

        if self.terms(vec![
            Tokens::Mul, Tokens::Div, Tokens::Mod
        ], true, Some(&me)) {
            self.adjust_single_child(me);
            return true;
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_sig_ops(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::SigOps);

        if self.terms(vec![
            Tokens::Negate, Tokens::Plus, Tokens::Minus
        ], true, Some(&me)) {
            self.adjust_single_child(me);
            return true;
        }

        if self.term_str(false, None).is_some() || 
            self.term_int(false, None).is_some() ||
            self.term_identity(false, None).is_some() ||
            self.term_decimal(false, None).is_some() ||
            self.term_bool(false, None).is_some() ||
            self.terms(vec![Tokens::Match, Tokens::LeftC], false, None) {
            self.current = cur;
            self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
            return true;
        }

        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_ops(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::Ops);

        'l: {
            if self.term(Tokens::Match, false, None) {
                if self.match_exec_match(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term_identity(false, None).is_some() {
                if self.match_exec_var(&me) {
                    self.adjust_single_child(me);
                    return true;
                }
                break 'l;
            }

            if self.term_str(true, Some(&me)).is_some() ||
                self.term_int(true, Some(&me)).is_some() ||
                self.term_decimal(true, Some(&me)).is_some() ||
                self.term_bool(true, Some(&me)).is_some() {
                self.adjust_single_child(me);
                return true;
            }

            // if self.terms(vec![
            //     Tokens::RightC, Tokens::OrS, Tokens::AndS,
            //     Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne,
            //     Tokens::Lt, Tokens::Ge, Tokens::Gt, Tokens::Le,
            //     Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div,
            //     Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            // ], false, None) {
            //     self.current = cur;
            //     self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
            //     return true;
            // }
        }

        println!("非法成员引用词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_stmt(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecStmt);

        if self.term(Tokens::Let, false, None) && 
            self.match_define_var(&me) {
            return true;
        }

        println!("成员声明内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_ret(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecRet);

        if self.term(Tokens::Return, true, None) && 
            self.match_exec_exp(&me) {
            return true;
        }

        println!("返回式内词法单元错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_break(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecBreak);

        if self.term(Tokens::Break, true, None) {
            return true;
        }
        
        println!("Break 式内词法单元错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_is(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecIs);

        if self.term_identity(false, None).is_some() && 
            self.match_exec_var(&me) && 
            self.match_exec_is_w(&me) && 
            self.match_exec_exp(&me) {
            return true;
        }

        println!("赋值语句内词法单元错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_is_w(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecIsW);

        if self.terms(vec![
            Tokens::PlusIs, Tokens::MinusIs, Tokens::DivIs, 
            Tokens::MulIs, Tokens::ModIs, Tokens::Is
        ], true, Some(&me)) {
            return true;
        }

        println!("非法赋值词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_match(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecMatch);

        if self.term(Tokens::Match, true, None) && 
            self.match_exec_exp(&me) &&
            self.term(Tokens::LeftBC, true, None) &&
            self.match_exec_match_s(&me) && 
            self.term(Tokens::RightBC, true, None) {
            return true;
        }

        println!("match 匹配式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }
    
    fn match_exec_match_s(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecMatchS);

        if (self.term_str(false, None).is_some() ||
            self.term_int(false, None).is_some() ||
            self.term_decimal(false, None).is_some() ||
            self.term_bool(false, None).is_some() ||
            self.term_identity(false, None).is_some() || 
            self.terms(vec![
                Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus
            ], false, None)) && 
                self.match_exec_exp(&me) &&
                self.term(Tokens::Semicolon, true, None) &&
                self.term(Tokens::LeftBC, true, None) && 
                self.match_fn_body(&me) &&
                self.term(Tokens::RightBC, true, None) && self.match_exec_match_e(&me) {
            return true;       
        }

        println!("match 匹配式域声明错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }
    
    fn match_exec_match_e(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecMatchE);

        'l: {
            if self.term(Tokens::Comma, true, None) {
                if self.match_exec_match_s(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightBC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("match 匹配式分割错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_if(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecIf);

        if self.term(Tokens::If, true, None) && 
            self.match_exec_exp(&me) &&
            self.term(Tokens::LeftBC, true, None) &&
            self.match_fn_body(&me) && 
            self.term(Tokens::RightBC, true, None) &&
            self.match_exec_if_e(&me) {
            return true;
        }

        println!("If 判断式错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_if_e(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecIfE);

        'l: {
            if self.term(Tokens::Else, true, Some(&me)) {
                if self.match_exec_if_ei(&me) && 
                    self.term(Tokens::LeftBC, true, None) &&
                    self.match_fn_body(&me) &&
                    self.term(Tokens::RightBC, true, None) &&
                    self.match_exec_if_e(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term_identity(false, None).is_some() ||
                self.term_str(false, None).is_some() ||
                self.term_int(false, None).is_some() ||
                self.term_decimal(false, None).is_some() ||
                self.term_bool(false, None).is_some() ||
                self.terms(vec![
                    Tokens::Let, Tokens::If, Tokens::While,
                    Tokens::LeftBC, Tokens::Return, Tokens::Break, 
                    Tokens::Match, Tokens::LeftC, Tokens::Negate, 
                    Tokens::Plus, Tokens::Minus, Tokens::RightBC
                ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("If 式 else 或 else if 式表示错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_if_ei(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecIfEi);

        'l: {
            if self.term(Tokens::If, true, Some(&me)) {
                if self.match_exec_exp(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::LeftBC, false, Some(&me)) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("else if 表示错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_while(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecWhile);

        if self.term(Tokens::While, true, None) && 
            self.match_exec_exp(&me) &&
            self.term(Tokens::LeftBC, true, None) &&
            self.match_fn_body(&me) && 
            self.term(Tokens::RightBC, true, None) {
            return true;
        }

        println!("While 式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_loop(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecLoop);

        if self.term(Tokens::LeftBC, true, None) && 
            self.match_fn_body(&me) &&
            self.term(Tokens::RightBC, true, None) && 
            self.term(Tokens::Loop, true, None) &&
            self.match_exec_exp(&me) {
            return true;
        }

        println!("Loop 式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_var(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecVar);

        if self.term_identity(true, Some(&me)).is_some() && 
            self.match_exec_var_t(&me) {
            return true;
        }

        println!("成员引用表达式内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_var_t(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecVarT);

        'l: {
            if self.term(Tokens::LeftMB, true, Some(&me)) {
                if self.term_int(true, Some(&me)).is_some() &&
                    self.term(Tokens::RightMB, true, Some(&me)) &&
                    self.match_exec_var_t(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::ShouldReturn, true, Some(&me)) {
                if self.match_exec_var_soe(&me) &&
                    self.match_exec_var_t(&me) {
                    return true;
                }
                break 'l;
            }            

            if self.term(Tokens::LeftC, true, Some(&me)) {
                if self.match_exec_func_p(&me) &&
                    self.term(Tokens::RightC, true, Some(&me)) &&
                    self.match_exec_var_t(&me) {
                    return true;
                }
                break 'l;
            }

            if self.terms(vec![
                Tokens::PlusIs, Tokens::MinusIs, Tokens::DivIs, Tokens::MulIs, Tokens::ModIs, Tokens::Is, Tokens::RightC, Tokens::OrS, Tokens::AndS, Tokens::Or, Tokens::And, Tokens::Eq, Tokens::Ne, Tokens::Gt, Tokens::Lt, Tokens::Ge, Tokens::Le, Tokens::Plus, Tokens::Minus, Tokens::Mul, Tokens::Div, Tokens::Mod, Tokens::Comma, Tokens::EndExp, Tokens::LeftBC, Tokens::Semicolon
            ], false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("复合结构、结构体引用、函数调用表示错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_var_soe(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecVarSoE);

        if self.term_int(true, Some(&me)).is_some() ||
            self.term_identity(true, Some(&me)).is_some() {
            return true;
        }

        println!("元组、结构体域引用错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_func_p(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecFuncP);

        'l: {
            if self.term_str(false, None).is_some() ||
                self.term_int(false, None).is_some() ||
                self.term_decimal(false, None).is_some() ||
                self.term_bool(false, None).is_some() ||
                self.term_identity(false, None).is_some() ||
                self.terms(vec![
                    Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus
                ], false, None) {
                if self.match_exec_func_params(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("函数调用内参数填写错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_func_params(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecFuncParams);

        if (self.term_str(false, None).is_some() ||
            self.term_int(false, None).is_some() ||
            self.term_decimal(false, None).is_some() ||
            self.term_bool(false, None).is_some() ||
            self.term_identity(false, None).is_some() || 
            self.terms(vec![
                Tokens::Match, Tokens::LeftC, Tokens::Negate, Tokens::Plus, Tokens::Minus
            ], false, None)) && 
            self.match_exec_exp(&me) && 
            self.match_exec_func_params_e(&me) {
            return true;
        }

        println!("函数调用参数表示错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_func_params_e(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecFuncParamsE);

        'l: {
            if self.term(Tokens::Comma, true, None) {
                if self.match_exec_func_params(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("函数调用参数连接表示错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_type(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecType);

        'l: {
            if self.term_identity(true, Some(&me)).is_some() {
                return true;
            }

            if self.term(Tokens::LeftMB, true, Some(&me)) {
                if self.match_exec_type(&me) &&
                    self.term(Tokens::EndExp, true, Some(&me)) &&
                    self.term_int(true, Some(&me)).is_some() &&
                    self.term(Tokens::RightMB, true, Some(&me)) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::LeftC, true, Some(&me)) {
                if self.match_exec_types_p(&me) &&
                    self.term(Tokens::RightC, true, Some(&me)) {
                    return true;       
                }
                break 'l;
            }
        }

        println!("类型声明错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_types_p(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecTypesP);

        'l: {
            if self.term_identity(false, None).is_some() || 
                self.terms(vec![
                    Tokens::LeftMB, Tokens::LeftC
                ], false, None) {
                if self.match_exec_types_params(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("元组类型声明错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_types_params(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecTypesParams);

        if self.match_exec_type(&me) &&
            self.match_exec_types_params_e(&me) {
            return true;
        }

        println!("元组类型域声明错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_exec_types_params_e(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::ExecTypesParamsE);

        'l: {
            if self.term(Tokens::Comma, true, None) {
                if self.match_exec_types_params(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("元组类型内类型连接错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_define_fn(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::DefineFn);

        if self.term(Tokens::Fn, true, None) && 
            self.term_identity(true, Some(&me)).is_some() &&
            self.term(Tokens::LeftC, true, None) && 
            self.match_fn_p(&me) &&
            self.term(Tokens::RightC, true, None) &&
            self.match_fn_return(&me) &&
            self.term(Tokens::LeftBC, true, None) &&
            self.match_fn_body(&me) && 
            self.term(Tokens::RightBC, true, None) {
            return true;
        }

        println!("函数定义内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_fn_p(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::FnP);

        'l: {
            if self.term_identity(false, None).is_some() {
                if self.match_fn_params(&me) {
                    return true;
                }
                break 'l;
            }
            
            if self.term(Tokens::RightC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("函数定义-参数表示错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_fn_params(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::FnParams);

        if self.term_identity(true, Some(&me)).is_some() && 
            self.term(Tokens::Semicolon, true, None) &&
            self.match_exec_type(&me) && self.match_fn_params_e(&me) {
            return true;
        }

        println!("函数定义-参数声明域内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_fn_params_e(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::FnParamsE);

        'l: {
            if self.term(Tokens::Comma, true, None) {
                if self.match_fn_params(&me) {
                    return true;
                }
                break 'l;
            }
            
            if self.term(Tokens::RightC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("函数定义-参数声明连接错误，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }

    fn match_fn_return(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::FnReturn);

        'l: {
            if self.term(Tokens::ShouldReturn, true, None) {
                if self.match_exec_type(&me) {
                    return true;
                }
                break 'l;
            }
            
            if self.term(Tokens::LeftBC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("函数定义-返回类型声明非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }
    
    fn match_fn_body(&mut self, root: &NodeId) -> bool {
        let cur = self.current;
        let me = insert_nt!(self.tree, root, NT::FnBody);

        'l: {
            if self.terms(vec![
                Tokens::Let, Tokens::If, 
                Tokens::While, Tokens::LeftBC, 
                Tokens::Return, Tokens::Break
            ], false, None) || 
                self.term_identity(false, None).is_some() {
                if self.match_exec_sentence(&me) &&
                    self.match_fn_body(&me) {
                    return true;
                }
                break 'l;
            }

            if self.term(Tokens::RightBC, false, None) {
                self.current = cur;
                self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
                return true;
            }
        }

        println!("函数定义-函数体内非法词法单元，目前词法单元：{:?}，位置: {:?}", self.tokens[self.current], self.current);
        self.current = cur;
        self.tree.remove_node(me, id_tree::RemoveBehavior::DropChildren).unwrap();
        false
    }
}

#[cfg(test)]
mod ll_parser_tests {
    use std::fs::File;

    use crate::lex::{preprocessor::preprocessor};

    use super::*;

    macro_rules! ll_parser_test_macro {
        ($file:expr, $test:expr) => {
            let file_path = format!("examples/sources/{}", $file);
            let mut path = std::env::current_dir().unwrap();
            path.push(file_path);
            let file = File::open(path).unwrap();
            let preprocess = preprocessor(&file);
            let analysis = Analysis::new_with_capacity($file, &preprocess, preprocess.len());
            let mut parser: RecursiveDescentParser = RecursiveDescentParser::new(analysis).unwrap();
            let r = parser.parse();
            parser.print_test();
            println!("current: {:?}", parser.get_current());
            if $test {
                assert!(r);
            } else {
                assert!(!r);
            }
        };
    }

    #[test]
    fn test1() {
        ll_parser_test_macro!("s8.ms", true);
    }

    #[test]
    fn test2() {
        ll_parser_test_macro!("s9.ms", true);
    }

    #[test]
    fn test3() {
        ll_parser_test_macro!("s10.ms", false);
    }

    #[test]
    fn test4() {
        ll_parser_test_macro!("s11.ms", true);
    }

    #[test]
    fn test5() {
        ll_parser_test_macro!("s12.ms", true);
    }

    #[test]
    fn test6() {
        ll_parser_test_macro!("s13.ms", true);
    }

    #[test]
    fn test7() {
        ll_parser_test_macro!("s14.ms", false);
    }

    #[test]
    fn test8() {
        ll_parser_test_macro!("s15.ms", true);
    }

    #[test]
    fn test9() {
        ll_parser_test_macro!("s16.ms", false);
    }

    #[test]
    fn test10() {
        ll_parser_test_macro!("s17.ms", true);
    }

    #[test]
    fn test11() {
        ll_parser_test_macro!("s18.ms", true);
    }

    #[test]
    fn test12() {
        ll_parser_test_macro!("s19.ms", true);
    }
}
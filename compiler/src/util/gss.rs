use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq)]
pub enum StackNode<T> {
    Root,
    Node { parent: Rc<StackNode<T>>, data: T }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Stack<T> {
    pub len: usize,
    pub tip: Rc<StackNode<T>>,
}

impl <T: Debug + Clone> Debug for Stack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_vec())
    }
}

#[derive(Debug)]
pub struct StackIter<T> {
    inner: Rc<StackNode<T>>,
}

impl <T: Clone + Eq + Debug> Stack<T> {
    pub fn eq_vec(&self, other: &Vec<T>) -> bool {
        &self.to_vec() == other
    }
}

impl <T: Clone> Stack<T> {
    pub fn empty() -> Stack<T> {
        Stack { len: 0, tip: Rc::new(StackNode::Root) }
    }

    pub fn from(stack: &Vec<T>) -> Stack<T> {
        let mut s = Stack { len: 0, tip: Rc::new(StackNode::Root) };
        for i in stack {
            s.push(i.clone());
        }
        s
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut vec = self.iter().collect::<Vec<T>>();
        vec.reverse();
        vec
    }

    pub fn push(&mut self, data: T) {
        // need to create a new stacknode
        self.tip = Rc::new(StackNode::Node { parent: self.tip.clone(), data });
        self.len += 1;
    }

    pub fn peek(&self) -> Option<T> {
        match self.tip.as_ref() {
            StackNode::Root => None,
            StackNode::Node { data, .. } => Some(data.clone())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if let StackNode::Node { parent, data } = self.tip.clone().as_ref() {
            self.tip = parent.clone();
            self.len -= 1;
            Some(data.clone())
        } else {
            None
        }
    }

    pub fn peek_n(&self, num: usize) -> Vec<T> {
        let mut v: Vec<T> = self.iter().take(num).collect();
        v.reverse();
        v
    }

    pub fn iter(&self) -> StackIter<T> {
        StackIter { inner: self.tip.clone() }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl <T: Clone> Iterator for StackIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.clone().as_ref() {
            StackNode::Root => None,
            StackNode::Node { parent, data } => {
                self.inner = parent.clone();
                Some(data.clone())
            }
        }
    }
}
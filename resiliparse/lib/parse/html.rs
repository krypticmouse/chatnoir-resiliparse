// Copyright 2023 Janek Bevendorff
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]

use std::{ptr, slice};
use std::ptr::addr_of_mut;
use std::rc::{Rc, Weak};

use crate::third_party::lexbor::*;

/// Internal heap-allocated and reference-counted HTMLTree.
struct HTMLTreeRc {
    html_document: *mut lxb_html_document_t
}

impl Drop for HTMLTreeRc {
    fn drop(&mut self) {
        if !self.html_document.is_null() {
            unsafe { lxb_html_document_destroy(self.html_document); }
            self.html_document = ptr::null_mut();
        }
    }
}

/// HTML DOM tree.
pub struct HTMLTree {
    tree_rc: Rc<HTMLTreeRc>
}

impl From<&[u8]> for HTMLTree {
    /// Decode a raw HTML byte string and parse it into a DOM tree.
    /// The bytes must be a valid UTF-8 encoding.
    fn from(value: &[u8]) -> Self {
        let doc_ptr;
        unsafe {
            doc_ptr = lxb_html_document_create();
            lxb_html_document_parse(doc_ptr, value.as_ptr(), value.len());
        }

        HTMLTree { tree_rc: Rc::new(HTMLTreeRc { html_document: doc_ptr }) }
    }
}

impl From<Vec<u8>> for HTMLTree {
    /// Decode a raw HTML byte string and parse it into a DOM tree.
    /// The bytes must be a valid UTF-8 encoding.
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        value.as_slice().into()
    }
}

impl From<&Vec<u8>> for HTMLTree {
    /// Decode a raw HTML byte string and parse it into a DOM tree.
    /// The bytes must be a valid UTF-8 encoding.
    #[inline]
    fn from(value: &Vec<u8>) -> Self {
        value.as_slice().into()
    }
}

impl From<&str> for HTMLTree {
    /// Parse HTML from a Unicode string slice into a DOM tree.
    #[inline]
    fn from(value: &str) -> Self {
        value.as_bytes().into()
    }
}

impl From<String> for HTMLTree {
    /// Parse HTML from a Unicode String into a DOM tree.
    #[inline]
    fn from(value: String) -> Self {
        value.as_bytes().into()
    }
}

impl From<&String> for HTMLTree {
    /// Parse HTML from a Unicode String into a DOM tree.
    #[inline]
    fn from(value: &String) -> Self {
        value.as_bytes().into()
    }
}

impl HTMLTree {
    fn get_html_document_raw(&self) -> Option<&mut lxb_html_document_t> {
        unsafe { self.tree_rc.html_document.as_mut() }
    }

    #[inline]
    pub fn document(&self) -> Option<DOMNode> {
        DOMNode::new(
            &self.tree_rc,
            addr_of_mut!(self.get_html_document_raw()?.dom_document) as *mut lxb_dom_node_t)
    }

    pub fn head(&self) -> Option<DOMNode> {
        DOMNode::new(&self.tree_rc, self.get_html_document_raw()?.head as *mut lxb_dom_node_t)
    }

    pub fn body(&self) -> Option<DOMNode> {
        DOMNode::new(&self.tree_rc, self.get_html_document_raw()?.body as *mut lxb_dom_node_t)
    }

    #[inline]
    pub fn title(&self) -> Option<String> {
        unsafe { Some(self.title_unsafe()?.to_owned()) }
    }

    pub unsafe fn title_unsafe(&self) -> Option<&str> {
        let mut title_len = 0;
        let cdata = lxb_html_document_title(self.get_html_document_raw()?, addr_of_mut!(title_len));
        match title_len {
            0 => None,
            _ => Some(std::str::from_utf8_unchecked(slice::from_raw_parts(cdata, title_len)))
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum NodeType {
    Element,
    Attribute,
    Text,
    CDataSection,
    EntityReference,
    Entity,
    ProcessingInstruction,
    Comment,
    Document,
    DocumentType,
    DocumentFragment,
    Notation,
    LastEntry,
    Undefined
}

impl From<lxb_dom_node_type_t> for NodeType {
    fn from(value: lxb_dom_node_type_t) -> Self {
        match value {
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_ELEMENT => NodeType::Element,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_ATTRIBUTE => NodeType::Attribute,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_TEXT => NodeType::Text,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_CDATA_SECTION => NodeType::CDataSection,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_ENTITY_REFERENCE => NodeType::EntityReference,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_ENTITY => NodeType::Entity,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_PROCESSING_INSTRUCTION => NodeType::ProcessingInstruction,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_COMMENT => NodeType::Comment,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_DOCUMENT => NodeType::Document,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_DOCUMENT_TYPE => NodeType::DocumentType,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_DOCUMENT_FRAGMENT => NodeType::DocumentFragment,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_NOTATION => NodeType::Notation,
            lxb_dom_node_type_t::LXB_DOM_NODE_TYPE_LAST_ENTRY => NodeType::LastEntry,
            _ => NodeType::Undefined,
        }
    }
}

/// DOM node.
pub struct DOMNode {
    tree: Weak<HTMLTreeRc>,
    node: *mut lxb_dom_node_t
}

impl DOMNode {
    #[inline]
    fn new(tree: &Rc<HTMLTreeRc>, node: *mut lxb_dom_node_t) -> Option<Self> {
        if node.is_null() {
            return None;
        }
        Some(Self { tree: Rc::downgrade(tree), node })
    }

    /// DOM node type.
    pub fn node_type(&self) -> NodeType {
        match self.tree.upgrade() {
            Some(_) => unsafe { (*self.node).type_.into() },
            _ => NodeType::Undefined
        }
    }

    /// DOM element tag or node name.
    pub fn tag(&self) -> Option<String> {
        unsafe { Some(self.tag_unsafe()?.to_owned()) }
    }

    /// DOM element tag or node name.
    pub unsafe fn tag_unsafe(&self) -> Option<&str> {
        self.tree.upgrade()?;
        unsafe {
            let mut size = 0;
            let name= lxb_dom_node_name(self.node, addr_of_mut!(size));
            match size {
                0 => None,
                _ => Some(std::str::from_utf8_unchecked(slice::from_raw_parts(name.cast(), size)))
            }
        }
    }

    /// First child element of this DOM node.
    pub fn first_child(&self) -> Option<Self> {
        unsafe { Self::new(&self.tree.upgrade()?, self.node.as_ref()?.first_child) }
    }

    /// Last child element of this DOM node.
    pub fn last_child(&self) -> Option<Self> {
        self.tree.upgrade()?;
        unsafe { Self::new(&self.tree.upgrade()?, self.node.as_ref()?.last_child) }
    }

    /// First element child of this DOM node.
    pub fn first_element_child(&self) -> Option<Self> {
        let mut child = self.first_child()?;
        loop {
            if child.node_type() == NodeType::Element {
                return Some(child);
            }
            child = child.next_sibling()?;
        }
    }

    /// Last element child element of this DOM node.
    pub fn last_element_child(&self) -> Option<Self> {
        let mut child = self.last_child()?;
        loop {
            if child.node_type() == NodeType::Element {
                return Some(child);
            }
            child = child.prev_sibling()?;
        }
    }

    /// List of child nodes.
    pub fn child_nodes(&self) -> Vec<Self> {
        let mut nodes = Vec::new();
        let mut child = self.first_child();
        while let Some(c) = child {
            child = c.next_sibling();
            nodes.push(c);
        }
        nodes
    }

    /// List of child element nodes.
    pub fn child_element_nodes(&self) -> Vec<Self> {
        let mut nodes = Vec::new();
        let mut child = self.first_element_child();
        while let Some(c) = child {
            child = c.next_element_sibling();
            nodes.push(c);
        }
        nodes
    }

    /// Parent of this node.
    pub fn parent(&self) -> Option<Self> {
        unsafe { Self::new(&self.tree.upgrade()?, self.node.as_ref()?.parent) }
    }

    /// Next sibling node.
    pub fn next_sibling(&self) -> Option<Self> {
        unsafe { Self::new(&self.tree.upgrade()?, self.node.as_ref()?.next) }
    }

    /// Previous sibling node.
    pub fn prev_sibling(&self) -> Option<Self> {
        unsafe { Self::new(&self.tree.upgrade()?, self.node.as_ref()?.prev) }
    }

    /// Next sibling element node.
    pub fn next_element_sibling(&self) -> Option<Self> {
        loop {
            let s = self.next_sibling()?;
            if s.node_type() == NodeType::Element {
                return Some(s);
            }
        }
    }

    /// Previous sibling element node.
    pub fn prev_element_sibling(&self) -> Option<Self> {
        loop {
            let s = self.prev_sibling()?;
            if s.node_type() == NodeType::Element {
                return Some(s);
            }
        }
    }

    /// Node text value.
    #[inline]
    pub fn value(&self) -> Option<String> {
        unsafe { Some(self.value_unsafe()?.to_owned()) }
    }

    /// Node text value.
    pub unsafe fn value_unsafe(&self) -> Option<&str> {
        self.tree.upgrade()?;
        let cdata = self.node as *const lxb_dom_character_data_t;
        Some(std::str::from_utf8_unchecked(slice::from_raw_parts(
            (*cdata).data.data.cast(), (*cdata).data.length)))
    }

    /// Text contents of this DOM node and its children.
    pub fn outer_text(&self) -> Option<String> {
        self.tree.upgrade()?;

        if self.node_type() == NodeType::Text {
            return self.value();
        }

        let out_text;
        unsafe {
            let mut l = 0;
            let t = lxb_dom_node_text_content(self.node, &mut l);
            out_text = std::str::from_utf8_unchecked(slice::from_raw_parts(t.cast(), l)).to_string();
            lxb_dom_document_destroy_text_noi(self.node.as_ref()?.owner_document, t);
        }
        Some(out_text)
    }

    /// Text contents of this DOM node and its children.
    #[inline]
    pub fn inner_text(&self) -> Option<String> {
        self.outer_text()
    }

    fn serialize_node(node: &Self) -> Option<String> {
        node.tree.upgrade()?;

        let out_html;
        unsafe {
            let h = lexbor_str_create();
            lxb_html_serialize_tree_str(node.node, h);
            out_html = std::str::from_utf8_unchecked(slice::from_raw_parts((*h).data.cast(), (*h).length)).to_string();
            lexbor_str_destroy(h, node.node.as_ref()?.owner_document.as_ref()?.text, true);
        }
        Some(out_html)
    }

    /// Outer HTML of this DOM node and its children.
    #[inline]
    pub fn outer_html(&self) -> Option<String> {
        Self::serialize_node(self)
    }

    /// Inner HTML of this DOM node's children.
    pub fn inner_html(&self) -> Option<String> {
        self.child_nodes()
            .into_iter()
            .flat_map(|c| Self::serialize_node(&c))
            .reduce(|a, b| a + &b)
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::html::HTMLTree;

    const HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Example page</title>
  </head>
  <body>
    <main id="foo">
      <p id="a">Hello <span class="bar">world</span>!</p>
      <p id="b" class="dom">Hello <a href="https://example.com" class="bar baz">DOM</a>!</p>
     </main>
     <!-- A comment -->
  </body>
</html>"#;

    #[test]
    fn parse_from_str() {
        let _tree1 = HTMLTree::from(HTML);
        let _tree2 = HTMLTree::from("<html></html>");
    }

    #[test]
    fn parse_from_string() {
        let _tree1 = HTMLTree::from(HTML.to_owned());
        let _tree2 = HTMLTree::from(&HTML.to_owned());
    }

    #[test]
    fn parse_from_bytes() {
        let _tree1 = HTMLTree::from(HTML.to_owned().into_bytes());
        let _tree2 = HTMLTree::from(&HTML.to_owned().into_bytes());
        let _tree3 = HTMLTree::from(HTML.as_bytes());
    }
}
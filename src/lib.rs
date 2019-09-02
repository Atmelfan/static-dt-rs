#![no_std]

//! # static-dt-rs
//!
//! `static-dt-rs` is a library to parse a static devicetree in an embedded environment without alloc.
//!

use crate::utils::{read_fdt_u32, get_fdt_string};

pub mod utils;

/// # Errors
/// Errors which can be returned by DeviceTree::new()
///
#[derive(Debug)]
pub enum Error {

    /// Magic does not match specification
    InvalidMagic,

    /// Last compatible version is not 16, contains read last compatible version
    UnsupportedVersion(u32),
}

/// # Tokens
/// FDT tokens that make up the structure of a devicetree
///
///
#[derive(Debug, Copy, Clone)]
pub enum Token<'a> {
    /// A token with an unknown or otherwise invalid ID.
    /// Shouldn't be returned
    Invalid(u32), // Shouldn't be returned

    /// Marks the beginning of a node
    ///
    /// Values:
    /// 1. devicetree
    /// 2. node offset
    /// 3. name
    ///
    /// Always have a matching EndNode
    BeginNode(&'a DeviceTree<'a>, usize, &'a [u8]),

    /// Marks the end of a node
    EndNode,

    /// Marks a property
    ///
    /// Values:
    /// 1. devicetree
    /// 2. name
    /// 3. data
    Property(&'a DeviceTree<'a>, &'a [u8], &'a [u8]),

    /// This token means nothing.
    NoOperation,

    /// Marks end of tokens
    End
}

impl<'a> Token<'a> {
    /// Returns a given name of this token or a representation
    ///
    pub fn name(&self) -> &'a [u8]{
        match self {
            Token::BeginNode(_, _, name) => name,
            Token::Property(_, name, _) => name,
            Token::EndNode => b"end-node",
            Token::NoOperation => b"nop",
            Token::End => b"end",
            _ => b"-"
        }
    }

    /// Return length of the node
    /// If token is a property, return its length in bytes
    /// If token is a node, return the number of properties and sub-nodes
    ///
    pub fn len(&self) -> usize{
        match self {
            /* If property, return its length in bytes */
            Token::Property(_, _, val) => val.len(),
            /* If node, return the number of properties and sub-nodes in it (single level) */
            Token::BeginNode(_, _, _) =>
                self.into_iter().filter(|x| match x {
                    Token::BeginNode(_,_,_) | Token::Property(_,_,_) => true,
                    _ => false
                }).count(),
            /* Not a property or node */
            _ => 0
        }
    }

    /// Returns true if len() equals 0. See len() for exact behaviour.
    pub fn empty(&self) -> bool {
        self.len() == 0
    }

    /// Read one byte from property at position n
    /// Returns None if not a property or out of range
    pub fn prop_u8(&self, n: usize) -> Option<u8>{
        match self {
            Token::Property(_, _, val) => {
                if n >= val.len() { return None }
                Some(val[n])
            },
            /* Not a property */
            _ => None
        }
    }

    /// Read one cell from property at position n
    /// Returns None if not a property or out of range
    pub fn prop_u32(&self, n: usize) -> Option<u32>{
        match self {
            Token::Property(_, _, val) => {
                if n*4+4 > val.len() { return None }
                Some(utils::read_fdt_u32(val, n*4))
            },
            /* Not a property */
            _ => None
        }
    }

    /// Read one string from start of property
    /// Returns None if not a property
    ///
    pub fn prop_str(&self) -> Option<&'a [u8]> {
        match self {
            Token::Property(_, _, val) => {
                utils::get_fdt_string(val, 0)
            },
            /* Not a property */
            _ => None
        }
    }

    /// Read one phandle (one cell) at position 0
    /// Returns None if token is not a property, out of range or failed to find a matching node
    pub fn prop_phandle(&self) -> Option<Token<'a>> {
        match self {
            Token::Property(dt,_, val) => {
                /* Try to read one cell and search for its node */
                match self.prop_u32(0) {
                    Some(phandle) => dt.get_phandle(phandle),
                    None => None //Out of range
                }
            },
            /* Not a property */
            _ => None
        }
    }

    /// Find a node with `name` in this node (not recursive)
    /// Returns None if there is no matching node.
    ///
    pub fn get_node(&self, name: &'a [u8]) -> Option<Token<'a>>{
        for tok in self.into_iter() {
            match tok {
                Token::BeginNode(_, _, s) => if name.eq(s) { return Some(tok) },
                _ => ()
            }
        }
        None
    }

    /// Find a property with `name` in this node (not recursive)
    /// Returns None if there is no matching property.
    ///
    pub fn get_prop(&self, name: &'a [u8]) -> Option<Token<'a>>{
        for tok in self.into_iter() {
            match tok {
                Token::Property(_, s, _) => if name.eq(s) { return Some(tok) },
                _ => ()
            }
        }
        None
    }

}

impl<'a> IntoIterator for Token<'a> {
    type Item = Token<'a>;
    type IntoIter = HierarchyTokenIterator<'a>;

    /// Returns a hierarchical iterator over this node
    /// Returns a empty iterator if token is not a node.
    ///
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Token::BeginNode(dt, offs, _) => HierarchyTokenIterator::new_offs(dt, offs),
            _ => HierarchyTokenIterator::none()
        }

    }
}

/// # TokenIterator
/// Iterates over FDT tokens (see Token) in a device tree.
/// Doesn't care about which level it's in.
pub struct TokenIterator<'a> {
    dt: Option<&'a DeviceTree<'a>>,
    offs: usize
}

impl<'a> TokenIterator<'a> {
    /// Create a new iterator over root
    fn new(dt: &'a DeviceTree<'a>) -> Self {
        TokenIterator { dt: Some(dt), offs: 0 }
    }

    /// Create a new iterator starting from offset, OFFSET MUST BE ALIGNED TO A TOKEN!
    fn new_offs(dt: &'a DeviceTree<'a>, offs: usize) -> Self {
        TokenIterator { dt: Some(dt), offs }
    }

    /// Create a empty iterator, will immediately return None
    fn none() -> Self {
        TokenIterator { dt: None, offs: 0 }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {

        match self.dt {
            Some(dt) => {
                /* Read token id */
                let token_id = read_fdt_u32(dt.structs, self.offs); self.offs += 4;

                match token_id {
                    1 => {
                        let s = get_fdt_string(dt.structs, self.offs).unwrap();
                        self.offs += (s.len()/4 + 1)*4;

                        Some(Token::BeginNode(dt, self.offs, s))
                    },
                    2 => Some(Token::EndNode),
                    3 => {
                        let len = read_fdt_u32(dt.structs, self.offs) as usize; self.offs += 4;
                        let nameoff = read_fdt_u32(dt.structs, self.offs) as usize; self.offs += 4;
                        let name = get_fdt_string(dt.strings, nameoff).unwrap();
                        let tmp = self.offs;
                        self.offs += ((len + 3) / 4)*4;
                        Some(Token::Property(dt, name, &dt.structs[tmp..tmp+(len as usize)]))
                    },
                    4 => Some(Token::NoOperation),
                    9 => None,
                    x => None //Some(Token::Invalid(x)) //useful for debugging
                }
            }
            None => None
        }
    }
}

/// # HierarchyTokenIterator
/// Iterates over FDT tokens but ignores token not in the current node
/// (i.e. between a node-begin and -end pair).
pub struct HierarchyTokenIterator<'a> {
    tokeniter: TokenIterator<'a>,
    level: i16
}

impl<'a> HierarchyTokenIterator<'a> {
    /// See `TokenIterator::new()`
    fn new(dt: &'a DeviceTree<'a>) -> Self {
        HierarchyTokenIterator {tokeniter: TokenIterator::new(dt), level: 0}
    }

    /// See `TokenIterator::new_offs()`
    fn new_offs(dt: &'a DeviceTree<'a>, offs: usize) -> Self {
        HierarchyTokenIterator{ tokeniter: TokenIterator::new_offs(dt, offs), level: 0 }
    }

    /// See `TokenIterator::none()`
    fn none() -> Self {
        HierarchyTokenIterator{ tokeniter: TokenIterator::none(), level: 0 }
    }
}

impl<'a> Iterator for HierarchyTokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {

        while let Some(tok) = self.tokeniter.next() {
            match tok {
                Token::BeginNode(_, _, _) => {
                    self.level += 1;
                    if self.level <= 1 { return Some(tok) }
                },
                Token::EndNode => {
                    self.level -= 1;
                    if self.level == 0 { return Some(tok) }
                    if self.level < 0 {return None}
                },
                _ => {
                    if self.level == 0 { return Some(tok) }
                }
            }
        }

        None

    }
}

/// The device tree
///
#[derive(Debug)]
pub struct DeviceTree<'a> {
    pub fdt: &'a [u8],

    pub structs: &'a [u8],
    pub strings: &'a [u8]
}

impl<'a> DeviceTree<'a> {

    /// Create a new DeviceTree with `fdt` as backing buffer.
    /// Returns Ok if header and version is correct. Respective Err() otherwise.
    ///
    pub fn back(fdt: &'a [u8]) -> Result<DeviceTree<'a>, Error> {

        let struct_offs = utils::read_fdt_u32(fdt, 8) as usize;
        let strings_offs = utils::read_fdt_u32(fdt, 12) as usize;
        let struct_size = utils::read_fdt_u32(fdt, 36) as usize;
        let string_size = utils::read_fdt_u32(fdt, 32) as usize;

        let dt = DeviceTree { fdt,
            structs: &fdt[struct_offs..struct_offs+struct_size],
            strings: &fdt[strings_offs..strings_offs+string_size]
        };

        /* Check the header */
        if dt.magic() != 0xD00DFEED_u32 {
            return Err(Error::InvalidMagic)
        }

        /* Check that the compatible version is 16 */
        if dt.last_comp_version() != 16 {
            return Err(Error::UnsupportedVersion(dt.last_comp_version()))
        }

        /* All ok */
        Ok(dt)
    }

    /// Returns the root node
    ///
    pub fn root(&self) -> Token {
        HierarchyTokenIterator::new(self).nth(0).unwrap()
    }

    /// Returns a iterator that will iterate over all tokens in the tree
    pub fn tokens(&self) -> TokenIterator{
        TokenIterator::new(self)
    }

    pub fn get_phandle(&self, phandle: u32) -> Option<Token> {
        /* zero is not a valid phandle */
        if phandle == 0 { return None; }

        let mut last_node = Token::Invalid(0);
        for token in self.tokens() {
            match token {
                Token::BeginNode(_,_,_) => {
                    last_node = token;
                },
                Token::Property(_,_,val) => {
                    match token.prop_u32(0) {
                        Some(x) => if x == phandle { return Some(last_node) }
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        None
    }

    /* Methods to access header information*/

    /// This field shall contain the value 0xd00dfeed (big-endian).
    pub fn magic(&self) -> u32 {
        utils::read_fdt_u32(self.fdt, 0)
    }

    /// This field shall contain the total size in bytes of the devicetree data structure. This size shall encompass all
    /// sections of the structure: the header, the memory reservation block, structure block and strings block, as well as any
    /// free space gaps between the blocks or after the final block.
    pub fn totalsize(&self) -> usize {
        utils::read_fdt_u32(self.fdt, 4) as usize
    }

    /// This field shall contain the version of the devicetree data structure. The version is 17 if using the structure as
    /// defined in this document. An DTSpec boot program may provide the devicetree of a later version, in which case
    /// this field shall contain the version number defined in whichever later document gives the details of that version.
    pub fn version(&self) -> u32 {
        utils::read_fdt_u32(self.fdt, 20)
    }

    /// This field shall contain the lowest version of the devicetree data structure with which the version
    /// used is backwards compatible. So, for the structure as defined in this document (version 17), this field shall contain
    /// 16 because version 17 is backwards compatible with version 16, but not earlier versions. As per section 5.1, a
    /// DTSpec boot program should provide a devicetree in a format which is backwards compatible with version 16, and
    /// thus this field shall always contain 16.
    pub fn last_comp_version(&self) -> u32 {
        utils::read_fdt_u32(self.fdt, 24)
    }

    /// This field shall contain the physical ID of the system’s boot CPU. It shall be identical to the
    /// physical ID given in the reg property of that CPU node within the devicetree.
    pub fn boot_cpuid_phys(&self) -> u32 {
        utils::read_fdt_u32(self.fdt, 28)
    }

}
use static_dt_rs::{DeviceTree, Token, HierarchyTokenIterator};
use static_dt_rs::utils::get_fdt_string;


static FDT: &[u8] = include_bytes!("test.dtb");

fn print_token(token: &Token) {
    match token {
        Token::BeginNode(buf, _, name) => {
            println!("node {}", String::from_utf8_lossy(name))
        },
        Token::EndNode => {
            println!("end-node")
        },
        Token::Invalid(id) => {
            println!("Invalid token {}", id)
        },
        Token::Property(buf, name, dat) => {
            println!("prop {}", String::from_utf8_lossy(name))
        },
        Token::NoOperation => {
            println!("nop")
        },
        _ => ()
    }
}

#[test]
fn parse_header() {

    let dt = DeviceTree::back(FDT).unwrap();

    assert_eq!(dt.version(), 17)
}

#[test]
fn parse_nodes() {

    let dt = DeviceTree::back(FDT).unwrap();

    println!("---- token iterator ----");
    let mut level = 0;
    for token in dt.tokens() {
        print_token(&token);
    }

    println!("---- hierarchy token iterator ----");
    for token in dt.root() {
        print_token(&token);
        match token {
            Token::BeginNode(_,_,_) => {
                println!(">>>>>>>>>>>>>");
                for tok in token {
                    print_token(&tok);
                }
                println!("<<<<<<<<<<<<<");
            },
            _ => ()
        }
    }

}

#[test]
fn test_len_prop() {
    let dt = DeviceTree::back(FDT).unwrap();
    let node1 = dt.root().get_node(b"node1").unwrap();

    /* Test propertis in node2*/
    let prop = node1.get_prop(b"a-byte-data-property").unwrap();
    assert_eq!(prop.len(), 4);
}

#[test]
fn test_len_node() {
    let dt = DeviceTree::back(FDT).unwrap();
    let node1 = dt.root().get_node(b"node1").unwrap();

    /* Test propertis in node2*/
    let prop = node1.get_node(b"child-node1").unwrap();
    assert_eq!(prop.len(), 4);
}

#[test]
fn test_prop_a_cell_property() {
    let dt = DeviceTree::back(FDT).unwrap();
    let node2 = dt.root().get_node(b"node2").unwrap();

    /* Test propertis in node2*/
    let prop = node2.get_prop(b"a-cell-property").unwrap();
    assert_eq!(prop.prop_u32(2).unwrap(), 3);
}

#[test]
fn test_prop_an_empty_property() {
    let dt = DeviceTree::back(FDT).unwrap();
    let node2 = dt.root().get_node(b"node2").unwrap();

    /* Test propertis in node2*/
    let prop = node2.get_prop(b"an-empty-property").unwrap();
    assert!(prop.empty());
}

#[test]
fn test_prop_a_string_property() {
    let dt = DeviceTree::back(FDT).unwrap();
    let node1 = dt.root().get_node(b"node1").unwrap();

    /* Test propertis in node2*/
    let prop = node1.get_prop(b"a-string-property").unwrap();
    assert_eq!(prop.prop_str().unwrap(), b"A string");
}

#[test]
fn test_phandle() {
    let dt = DeviceTree::back(FDT).unwrap();
    let node2 = dt.root().get_node(b"node2").unwrap();

    /* a-phandle-property points to '/node1/child-node1' */
    let phandle_prop = node2.get_prop(b"a-phandle-property").unwrap();

    /* prop_phandle() reads one cell and tries to find a matching phandle, returning None if unsuccessful */
    let phandle_node = phandle_prop.prop_phandle().unwrap();

    /* Verify that phandle_node is '/node1/child-node1'*/
    let prop = phandle_node.get_prop(b"a-string-property").unwrap();
    assert_eq!(prop.prop_str().unwrap(), b"Hello, world");
}
use static_dt_rs::{DeviceTree, Token};

static FDT: &[u8] = include_bytes!("test.dtb");

fn main() {
    let dt = DeviceTree::back(FDT).unwrap();

    let root = dt.root();

    let node2 = root.get_node(b"node2").unwrap();

    let prop = node2.get_prop(b"a-cell-property").unwrap();

    println!("/node2/a-cell-property.len = {}",  prop.len());
    for x in 0..5 {
        println!("/node2/a-cell-property.{} = {}", x, prop.prop_u32(x).unwrap_or_default());
    }

    if let Some(node_dont_exist) = root.get_node(b"node-i-dont-exist") {
        println!("/node_dont_exist.len = {}",  node_dont_exist.len());
    }else{
        println!("/node_dont_exist doesn't exist!");
    }

    if let Some(node1) = root.get_node(b"node1") {
        println!("/node1.len = {}",  node1.len());

        for token in node1 {
            match token {
                Token::BeginNode(_,_,name) | Token::Property(_,name,_) => {
                    println!("/node1/{}", String::from_utf8_lossy(name));
                },
                _ => ()
            }
        }

    }else{
        println!("/node1 doesn't exist!");
    }



}


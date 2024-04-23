use crate::scene::bvh::Node;
use crate::scene::Triangle;
pub struct Tree {
  nodes: Vec<Node>,
  triangles: Vec<Triangle>,
}

impl Tree {
  fn new(mesh: &Mesh) -> Self {
    let mut triangles = Triangle::triangulate(mesh);
    triangles.sort_by(|a,b| a.partial_cmp(&b).unwrap());
    let n = triangles.len();
    let nodes = vec![Node::default();n];
    for i in 0..n {
      let mut j = (i+n)/2;
      while j > 0 {
        nodes[j].refit(triangles[i]);
        j /= 2;
      }
    }
    Self {
      triangles,
      nodes,
    }
  }
}

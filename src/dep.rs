use crate::parse::{ParsedLay, Sprite, SpriteT};
use crate::raise;
use crate::SgSpriteErr;

pub struct DepNode<'a> {
    pub sprite: &'a Sprite,
    pub ref_count: usize,
    pub depends_on: Option<usize>,
}

pub struct DepGraph<'a>(Vec<DepNode<'a>>);

fn node(s: &Sprite) -> DepNode {
    DepNode { sprite: s, ref_count: 0, depends_on: None }
}

impl<'g> DepGraph<'g> {
    pub fn resolve_dep_graph(lay: &ParsedLay) -> DepGraph {
        let base_dep = lay.base_dep;
        let mut node_list: Vec<_> = lay.sprites.iter().map(node).collect();

        // Dep: depend on Base/first sprite
        //      implicitly if parent is absent

        for i in 0..node_list.len() {
            let n = &mut node_list[i]; // No way to modify other nodes while iterator is borrowed
            let dep_idx = match n.sprite.sprite_type {
                SpriteT::Base | SpriteT::Overlay => continue,
                SpriteT::Sub => base_dep,
                SpriteT::Dep { depends_on: dep, .. } => match lay.sub_map.get(&dep) {
                    Some(d) => Some(*d),
                    None => base_dep,
                },
            };

            if let Some(i) = dep_idx {
                n.depends_on = dep_idx;
                node_list[i].ref_count += 1;
            }
        }

        DepGraph(node_list)
    }

    pub fn get_leaf_sprites<'a>(&'a self) -> impl Iterator<Item = &'a DepNode<'g>> {
        self.0.iter().filter(|d| d.ref_count == 0)
    }

    pub fn resolve_layers(&self, leaf: &'g DepNode) -> Result<Vec<&'g Sprite>, SgSpriteErr> {
        let node_count = self.0.len();
        let mut layers: Vec<&Sprite> = Vec::with_capacity(3);

        let mut next = Some(leaf);
        while let Some(s) = next {
            layers.push(s.sprite);
            next = s.depends_on.map(|d| &self.0[d]);
            if layers.len() > node_count {
                raise!("sprite layer list resolve looped");
            }
        }

        Ok(layers)
    }
}

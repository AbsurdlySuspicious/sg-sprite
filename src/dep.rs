use crate::parse::{ParsedLay, Sprite, SpriteT};
use crate::{raise, PErr};

pub struct DepRef<'a> {
    pub idx: usize,
    pub s: &'a Sprite,
    pub rc: usize,
    pub dep_on: Option<usize>,
}

pub fn resolve_rc(lay: &ParsedLay) -> Vec<DepRef> {
    let base_dep = lay.base_dep;
    let mut lst: Vec<_> = lay
        .sprites
        .iter()
        .enumerate()
        .map(|(i, s)| DepRef {
            s,
            idx: i,
            rc: 0,
            dep_on: None,
        })
        .collect();

    // Dep: depend on Base/first sprite
    //      implicitly if parent is absent

    for dr_i in 0..lst.len() {
        let dr = &mut lst[dr_i];
        let dep_i = match dr.s.sprite_type {
            SpriteT::Base => continue,
            SpriteT::Overlay => continue,
            SpriteT::Sub => base_dep,
            SpriteT::Dep { depends_on: dep, .. } => match lay.sub_map.get(&dep) {
                Some(d) => Some(*d),
                None => base_dep,
            },
        };

        if let Some(i) = dep_i {
            dr.dep_on = dep_i;
            lst[i].rc += 1;
        }
    }

    lst
}

pub fn leaf_sprites<'a, 'b>(v: &'a [DepRef<'b>]) -> impl Iterator<Item = &'a DepRef<'b>> {
    v.iter().filter(|d| d.rc == 0 /*&& d.s.t != SpriteT::Overlay*/)
}

pub fn overlays<'a, 'b>(v: &'a [DepRef<'b>]) -> impl Iterator<Item = &'a DepRef<'b>> {
    v.iter().filter(|d| d.s.sprite_type == SpriteT::Overlay)
}

pub fn overlay_on_leaf<'a>(ovr: &DepRef<'a>, leaf: &DepRef<'a>) -> DepRef<'a> {
    assert_eq!(ovr.s.sprite_type, SpriteT::Overlay);
    DepRef {
        dep_on: Some(leaf.idx),
        ..*ovr
    }
}

pub fn resolve_dep_list<'a>(v: &[DepRef<'a>], src: &'a DepRef) -> Result<Vec<&'a Sprite>, PErr> {
    let v_ln = v.len();
    let mut dep_q: Vec<&Sprite> = Vec::with_capacity(3);

    let mut next = Some(src);
    while let Some(s) = next {
        dep_q.push(s.s);
        next = s.dep_on.map(|d| &v[d]);
        if dep_q.len() > v_ln {
            return raise("dep-list resolve looped");
        }
    }

    Ok(dep_q)
}

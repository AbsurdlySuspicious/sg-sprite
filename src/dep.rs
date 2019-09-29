use crate::parse::{ParsedLay, Sprite, SpriteT::*};
use crate::PErr;

pub struct DepRef<'a> {
  pub s: &'a Sprite,
  pub rc: usize,
  pub dep_on: Option<usize>,
}

pub fn resolve_rc(lay: &ParsedLay) -> Vec<DepRef> {
  let mut lst: Vec<_> = lay
    .sprites
    .iter()
    .map(|s| DepRef {
      s,
      rc: 0,
      dep_on: None,
    })
    .collect();

  for dr_i in 0..lst.len() {
    let dr = &mut lst[dr_i];
    let dep_i = match dr.s.t {
      Base => continue,
      Sub => 0usize,
      Dep(n) => *lay.sub_map.get(&n).unwrap_or(&0),
    };

    dr.dep_on = Some(dep_i);
    lst[dep_i].rc += 1;
  }

  lst
}

pub fn leaf_sprites<'a, 'b>(v: &'a Vec<DepRef<'b>>) -> impl Iterator<Item = &'a DepRef<'b>> {
  v.iter().filter(|d| d.rc == 0)
}

pub fn resolve_dep_list<'a>(v: &Vec<DepRef<'a>>, src: &'a DepRef) -> Vec<&'a Sprite> {
  let mut dep_q: Vec<&Sprite> = Vec::with_capacity(3);

  let mut next = Some(src);
  while let Some(s) = next {
    dep_q.push(s.s);
    next = s.dep_on.map(|d| &v[d]);
  }

  dep_q
}

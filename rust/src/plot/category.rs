use std::borrow::BorrowMut;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Category {
    pub label: String,
    pub members: Vec<String>,
    pub indices: Vec<usize>,
    pub color: String,
}

pub fn set_cat_order(
    values: &Vec<(String, usize)>,
    order: &Option<String>,
    count: &usize,
    palette: &Vec<String>,
) -> (Vec<Category>, Vec<usize>) {
    let mut indices = HashMap::new();
    let mut label_list = vec![];
    for (i, entry) in values.iter().enumerate() {
        label_list.push(entry.clone().0);
        if !indices.contains_key(&entry.0) {
            indices.insert(entry.clone().0, vec![i]);
        } else {
            let list = indices.get_mut(&entry.0).unwrap();
            list.push(i);
        }
    }
    let frequencies = values
        .iter()
        .map(|x| x.clone().0)
        .fold(HashMap::new(), |mut map, val| {
            map.entry(val).and_modify(|frq| *frq += 1).or_insert(1);
            map
        });
    let mut sorted_cats: Vec<_> = frequencies.clone().into_iter().collect();
    sorted_cats.sort_by(|x, y| {
        if x.1 == y.1 {
            x.0.partial_cmp(&y.0).unwrap()
        } else {
            y.1.cmp(&x.1)
        }
    });

    let mut cat_order = vec![];
    let mut index = 0;
    if order.is_some() {
        // TODO: prevent duplication when adding remaining cats
        for entry in order.clone().unwrap().split(",") {
            if frequencies.contains_key(entry) {
                cat_order.push(Category {
                    label: entry.to_string(),
                    members: vec![],
                    indices: vec![],
                    color: palette[index].clone(),
                });
                index += 1;
            }
        }
    }
    for (label, _) in &sorted_cats {
        if index < count - 1 || index == count - 1 && *count == sorted_cats.len() {
            cat_order.push(Category {
                label: label.clone(),
                members: vec![label.clone()],
                indices: indices[label].clone(),
                color: palette[index].clone(),
            });
            index += 1
        } else if cat_order.len() < *count {
            cat_order.push(Category {
                label: "other".to_string(),
                members: vec![label.clone()],
                indices: indices[label].clone(),
                color: palette[count - 1].clone(),
            });
        } else {
            let other_cat = cat_order[count - 1].borrow_mut();
            other_cat.members.push(label.clone());
            other_cat.indices.append(indices.get_mut(label).unwrap());
        }
    }
    let mut cat_indices: Vec<usize> = (0..values.len()).collect();
    for (index, cat) in cat_order.iter().enumerate() {
        for i in cat.indices.iter() {
            cat_indices[*i] = index;
        }
    }
    (cat_order, cat_indices)
}

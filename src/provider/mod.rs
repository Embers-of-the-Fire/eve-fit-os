use std::collections::HashMap;

use crate::fit;

pub trait InfoProvider {
    fn get_dogma_attributes(&self, type_id: i32) -> Vec<fit::TypeDogmaAttribute>;
    fn get_dogma_attribute(&self, attribute_id: i32) -> &fit::DogmaAttribute;
    fn get_dogma_effects(&self, type_id: i32) -> Vec<fit::TypeDogmaEffect>;
    fn get_dogma_effect(&self, effect_id: i32) -> &fit::DogmaEffect;
    fn get_buff(&self, buff_id: i32) -> &fit::Buff;
    fn get_type(&self, type_id: i32) -> &fit::Type;
}

pub trait FitProvider {
    fn skills(&self) -> &HashMap<i32, u8>;
    fn fit(&self) -> &fit::ItemFit;

    fn get_dynamic_item_base_type_id(&self, dynamic_item_id: i32) -> i32;
    fn get_dynamic_item(&self, dynamic_item_id: i32) -> &fit::DynamicItem;
}

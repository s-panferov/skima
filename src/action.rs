use downcast_rs::Downcast;

pub trait Action: Downcast + std::fmt::Debug {}

downcast_rs::impl_downcast!(Action);

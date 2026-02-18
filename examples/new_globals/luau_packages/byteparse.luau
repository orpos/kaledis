local module = require("./.pesde/plainenglish+byteparse/1.2.1/byteparse/lib")
export type u8  = module.u8 
export type u16  = module.u16 
export type u32  = module.u32 
export type i8  = module.i8 
export type i16  = module.i16 
export type i32  = module.i32 
export type f32  = module.f32 
export type f64  = module.f64 
export type varint  = module.varint 
export type Endianness  = module.Endianness 
export type Serialisable  = module.Serialisable 
export type SerialisableArray  = module.SerialisableArray 
export type SerialisableTable  = module.SerialisableTable 
export type DataType  = module.DataType 
return module
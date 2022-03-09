use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.utils.CompressionAlgorithm`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.utils.CompressionAlgorithm`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.utils"), "CompressionAlgorithm"),
        None,
        Method::from_builtin(
            instance_init,
            "<CompressionAlgorithm instance initializer>",
            mc,
        ),
        Method::from_builtin(class_init, "<CompressionAlgorithm class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] =
        &[("DEFLATE", "deflate"), ("LZMA", "lzma"), ("ZLIB", "zlib")];
    write.define_public_constant_string_class_traits(CONSTANTS);

    class
}

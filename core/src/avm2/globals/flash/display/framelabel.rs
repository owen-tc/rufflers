//! `flash.display.FrameLabel` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::globals::NS_RUFFLE_INTERNAL;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.FrameLabel`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let name = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(activation)?;
    let frame = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_i32(activation)?;

    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        this.set_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "name").into(),
            name.into(),
            activation,
        )?;
        this.set_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "frame").into(),
            frame.into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.FrameLabel`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `FrameLabel.name`.
pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        return this.get_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "name").into(),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `FrameLabel.frame`.
pub fn frame<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        return this.get_property(
            &QName::new(Namespace::Private(NS_RUFFLE_INTERNAL.into()), "frame").into(),
            activation,
        );
    }

    Ok(Value::Undefined)
}

/// Construct `FrameLabel`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "FrameLabel"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init, "<FrameLabel instance initializer>", mc),
        Method::from_builtin(class_init, "<FrameLabel class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("name", Some(name), None), ("frame", Some(frame), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PRIVATE_INSTANCE_SLOTS: &[(&str, &str, &str, &str)] = &[
        (NS_RUFFLE_INTERNAL, "name", "", "String"),
        (NS_RUFFLE_INTERNAL, "frame", "", "int"),
    ];
    write.define_private_slot_instance_traits(PRIVATE_INSTANCE_SLOTS);

    class
}

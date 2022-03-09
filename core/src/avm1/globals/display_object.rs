//! DisplayObject common methods

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::display_object::{DisplayObject, Lists, TDisplayObject, TDisplayObjectContainer};
use crate::string::AvmString;
use gc_arena::MutationContext;

/// Depths used/returned by ActionScript are offset by this amount from depths used inside the SWF/by the VM.
/// The depth of objects placed on the timeline in the Flash IDE start from 0 in the SWF,
/// but are negative when queried from MovieClip.getDepth().
/// Add this to convert from AS -> SWF depth.
pub const AVM_DEPTH_BIAS: i32 = 16384;

/// The maximum depth that the AVM will allow you to swap or attach clips to.
/// What is the derivation of this number...?
pub const AVM_MAX_DEPTH: i32 = 2_130_706_428;

/// The maximum depth that the AVM will allow you to remove clips from.
/// What is the derivation of this number...?
pub const AVM_MAX_REMOVE_DEPTH: i32 = 2_130_706_416;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "getDepth" => method(get_depth; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "_global" => property(get_global, overwrite_global; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "_root" => property(get_root, overwrite_root; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "_parent" => property(get_parent, overwrite_parent; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// Add common display object prototype methods to the given prototype.
pub fn define_display_object_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    object: ScriptObject<'gc>,
    fn_proto: Object<'gc>,
) {
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
}

pub fn get_global<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.avm1.global_object())
}

pub fn get_root<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.root_object()
}

pub fn get_parent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_display_object()
        .and_then(|mc| mc.avm1_parent())
        .map(|dn| dn.object().coerce_to_object(activation))
        .map(Value::Object)
        .unwrap_or(Value::Undefined))
}

pub fn get_depth<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        if activation.swf_version() >= 6 {
            let depth = display_object.depth().wrapping_sub(AVM_DEPTH_BIAS);
            return Ok(depth.into());
        }
    }
    Ok(Value::Undefined)
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(display_object) = this.as_display_object() {
        Ok(AvmString::new(activation.context.gc_context, display_object.path()).into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn overwrite_root<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(
        activation.context.gc_context,
        "_root",
        new_val,
        Attribute::empty(),
    );

    Ok(Value::Undefined)
}

pub fn overwrite_global<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(
        activation.context.gc_context,
        "_global",
        new_val,
        Attribute::empty(),
    );

    Ok(Value::Undefined)
}

pub fn overwrite_parent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(
        activation.context.gc_context,
        "_parent",
        new_val,
        Attribute::empty(),
    );

    Ok(Value::Undefined)
}

pub fn remove_display_object<'gc>(
    this: DisplayObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) {
    let depth = this.depth().wrapping_sub(0);
    // Can only remove positive depths (when offset by the AVM depth bias).
    // Generally this prevents you from removing non-dynamically created clips,
    // although you can get around it with swapDepths.
    // TODO: Figure out the derivation of this range.
    if depth >= AVM_DEPTH_BIAS && depth < AVM_MAX_REMOVE_DEPTH && !this.removed() {
        // Need a parent to remove from.
        if let Some(mut parent) = this.avm1_parent().and_then(|o| o.as_movie_clip()) {
            parent.remove_child(&mut activation.context, this, Lists::all());
        }
    }
}

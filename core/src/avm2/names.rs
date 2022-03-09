//! AVM2 names & namespacing

use crate::avm2::activation::Activation;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::{AvmString, WStr, WString};
use gc_arena::{Collect, MutationContext};
use swf::avm2::types::{
    AbcFile, Index, Multiname as AbcMultiname, Namespace as AbcNamespace,
    NamespaceSet as AbcNamespaceSet,
};

/// Represents the name of a namespace.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub enum Namespace<'gc> {
    Namespace(AvmString<'gc>),
    Package(AvmString<'gc>),
    PackageInternal(AvmString<'gc>),
    Protected(AvmString<'gc>),
    Explicit(AvmString<'gc>),
    StaticProtected(AvmString<'gc>),
    Private(AvmString<'gc>),
    Any,
}

impl<'gc> Namespace<'gc> {
    /// Read a namespace declaration from the ABC constant pool and copy it to
    /// a namespace value.
    pub fn from_abc_namespace(
        translation_unit: TranslationUnit<'gc>,
        namespace_index: Index<AbcNamespace>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        if namespace_index.0 == 0 {
            return Ok(Self::Any);
        }

        let actual_index = namespace_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_namespace: Result<_, Error> = abc
            .constant_pool
            .namespaces
            .get(actual_index)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        Ok(match abc_namespace? {
            AbcNamespace::Namespace(idx) => {
                Self::Namespace(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Package(idx) => Self::Package(translation_unit.pool_string(idx.0, mc)?),
            AbcNamespace::PackageInternal(idx) => {
                Self::PackageInternal(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Protected(idx) => {
                Self::Protected(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Explicit(idx) => Self::Explicit(translation_unit.pool_string(idx.0, mc)?),
            AbcNamespace::StaticProtected(idx) => {
                Self::StaticProtected(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Private(idx) => Self::Private(translation_unit.pool_string(idx.0, mc)?),
        })
    }

    pub fn public() -> Self {
        Self::Package("".into())
    }

    pub fn as3_namespace() -> Self {
        Self::Namespace("http://adobe.com/AS3/2006/builtin".into())
    }

    pub fn package(package_name: impl Into<AvmString<'gc>>) -> Self {
        Self::Package(package_name.into())
    }

    pub fn internal(package_name: impl Into<AvmString<'gc>>) -> Self {
        Self::PackageInternal(package_name.into())
    }

    pub fn private(name: impl Into<AvmString<'gc>>) -> Self {
        Self::Private(name.into())
    }

    pub fn is_public(&self) -> bool {
        matches!(self, Self::Package(name) if name.is_empty())
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }

    pub fn is_package(&self, package_name: impl Into<AvmString<'gc>>) -> bool {
        if let Self::Package(my_name) = self {
            return my_name == &package_name.into();
        }

        false
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace(_))
    }

    /// Get the string value of this namespace, ignoring its type.
    ///
    /// TODO: Is this *actually* the namespace URI?
    pub fn as_uri(&self) -> AvmString<'gc> {
        match self {
            Self::Namespace(s) => *s,
            Self::Package(s) => *s,
            Self::PackageInternal(s) => *s,
            Self::Protected(s) => *s,
            Self::Explicit(s) => *s,
            Self::StaticProtected(s) => *s,
            Self::Private(s) => *s,
            Self::Any => "".into(),
        }
    }
}

/// A `QName`, likely "qualified name", consists of a namespace and name string.
///
/// This is technically interchangeable with `xml::XMLName`, as they both
/// implement `QName`; however, AVM2 and XML have separate representations.
///
/// A property cannot be retrieved or set without first being resolved into a
/// `QName`. All other forms of names and multinames are either versions of
/// `QName` with unspecified parameters, or multiple names to be checked in
/// order.
#[derive(Clone, Copy, Collect, Debug, Hash)]
#[collect(no_drop)]
pub struct QName<'gc> {
    ns: Namespace<'gc>,
    name: AvmString<'gc>,
}

impl<'gc> PartialEq for QName<'gc> {
    fn eq(&self, other: &Self) -> bool {
        // Implemented by hand to enforce order of comparisons for perf
        self.name == other.name && self.ns == other.ns
    }
}

impl<'gc> Eq for QName<'gc> {}

impl<'gc> QName<'gc> {
    pub fn new(ns: Namespace<'gc>, name: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns,
            name: name.into(),
        }
    }

    pub fn dynamic_name(local_part: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns: Namespace::public(),
            name: local_part.into(),
        }
    }

    /// Pull a `QName` from the multiname pool.
    ///
    /// This function returns an Err if the multiname does not exist or is not
    /// a `QName`.
    pub fn from_abc_multiname(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        if multiname_index.0 == 0 {
            return Err("Attempted to load a trait name of index zero".into());
        }

        let actual_index = multiname_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } => Self {
                ns: Namespace::from_abc_namespace(translation_unit, *namespace, mc)?,
                name: translation_unit.pool_string(name.0, mc)?,
            },
            _ => return Err("Attempted to pull QName from non-QName multiname".into()),
        })
    }

    /// Constructs a `QName` from a fully qualified name.
    ///
    /// A fully qualified name can be any of the following formats:
    /// NAMESPACE::LOCAL_NAME
    /// NAMESPACE.LOCAL_NAME (Where the LAST dot is used to split the namespace & local_name)
    /// LOCAL_NAME (Use the public namespace)
    pub fn from_qualified_name(name: AvmString<'gc>, mc: MutationContext<'gc, '_>) -> Self {
        let parts = name
            .rsplit_once(WStr::from_units(b"::"))
            .or_else(|| name.rsplit_once(WStr::from_units(b".")));

        if let Some((package_name, local_name)) = parts {
            Self {
                ns: Namespace::Package(AvmString::new(mc, package_name)),
                name: AvmString::new(mc, local_name),
            }
        } else {
            Self {
                ns: Namespace::public(),
                name,
            }
        }
    }

    /// Converts this `QName` to a fully qualified name.
    pub fn to_qualified_name(self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let uri = self.namespace().as_uri();
        let name = self.local_name();
        uri.is_empty().then(|| name).unwrap_or_else(|| {
            let mut buf = WString::from(uri.as_wstr());
            buf.push_str(WStr::from_units(b"::"));
            buf.push_str(&name);
            AvmString::new(mc, buf)
        })
    }

    pub fn local_name(&self) -> AvmString<'gc> {
        self.name
    }

    pub fn namespace(self) -> Namespace<'gc> {
        self.ns
    }

    /// Get the string value of this QName, including the namespace URI.
    pub fn as_uri(&self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let ns = match &self.ns {
            Namespace::Namespace(s) => s,
            Namespace::Package(s) => s,
            Namespace::PackageInternal(s) => s,
            Namespace::Protected(s) => s,
            Namespace::Explicit(s) => s,
            Namespace::StaticProtected(s) => s,
            Namespace::Private(s) => s,
            Namespace::Any => WStr::from_units(b"*"),
        };

        if ns.is_empty() {
            return self.name;
        }

        let mut uri = WString::from(ns);
        uri.push_str(WStr::from_units(b"::"));
        uri.push_str(&self.name);
        AvmString::new(mc, uri)
    }
}

/// A `Multiname` consists of a name which could be resolved in one or more
/// potential namespaces.
///
/// All unresolved names are of the form `Multiname`, and the name resolution
/// process consists of searching each name space for a given name.
///
/// The existence of a `name` of `None` indicates the `Any` name.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Multiname<'gc> {
    /// The list of namespaces that satisfy this multiname.
    ns: Vec<Namespace<'gc>>,

    /// The local name that satisfies this multiname. If `None`, then this
    /// multiname is satisfied by any name in the namespace.
    name: Option<AvmString<'gc>>,

    /// The type parameters required to satisfy this multiname. If empty, then
    /// this multiname is satisfied by any type parameters in any amount.
    params: Vec<Multiname<'gc>>,
}

impl<'gc> Multiname<'gc> {
    /// Read a namespace set from the ABC constant pool, and return a list of
    /// copied namespaces.
    fn abc_namespace_set(
        translation_unit: TranslationUnit<'gc>,
        namespace_set_index: Index<AbcNamespaceSet>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Vec<Namespace<'gc>>, Error> {
        if namespace_set_index.0 == 0 {
            //TODO: What is namespace set zero?
            return Ok(vec![]);
        }

        let actual_index = namespace_set_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let ns_set: Result<_, Error> = abc
            .constant_pool
            .namespace_sets
            .get(actual_index)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let mut result = vec![];

        for ns in ns_set? {
            result.push(Namespace::from_abc_namespace(translation_unit, *ns, mc)?)
        }

        Ok(result)
    }

    /// Assemble a multiname from an ABC `MultinameL` and the late-bound name.
    ///
    /// Intended for use by code that wants to inspect the late-bound name's
    /// value first before using standard namespace lookup.
    pub fn from_multiname_late(
        translation_unit: TranslationUnit<'gc>,
        abc_multiname: &AbcMultiname,
        name: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        match abc_multiname {
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => Ok(Self {
                ns: Self::abc_namespace_set(
                    translation_unit,
                    *namespace_set,
                    activation.context.gc_context,
                )?,
                name: Some(name.coerce_to_string(activation)?),
                params: Vec::new(),
            }),
            _ => Err("Cannot assemble early-bound multinames using from_multiname_late".into()),
        }
    }

    /// Resolve an ABC multiname's parameters and yields an AVM multiname with
    /// those parameters filled in.
    ///
    /// This function deliberately errors out if handed a `TypeName`, as it
    /// assumes that this is an attempt to construct a recursive generic type.
    /// Type parameters may themselves be typenames, but not the base type.
    /// This is valid: `Vector.<Vector.<int>>`, but this is not:
    /// `Vector.<int>.<int>`
    fn resolve_multiname_params(
        translation_unit: TranslationUnit<'gc>,
        abc_multiname: &AbcMultiname,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        Ok(match abc_multiname {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(
                        translation_unit,
                        *namespace,
                        activation.context.gc_context,
                    )?],
                    name: translation_unit
                        .pool_string_option(name.0, activation.context.gc_context)?,
                    params: Vec::new(),
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => {
                let ns_value = activation.avm2().pop();
                let ns = ns_value.as_namespace()?;
                Self {
                    ns: vec![*ns],
                    name: translation_unit
                        .pool_string_option(name.0, activation.context.gc_context)?,
                    params: Vec::new(),
                }
            }
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => {
                let name = activation.avm2().pop().coerce_to_string(activation)?;
                let ns_value = activation.avm2().pop();
                let ns = ns_value.as_namespace()?;
                Self {
                    ns: vec![*ns],
                    name: Some(name),
                    params: Vec::new(),
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(
                    translation_unit,
                    *namespace_set,
                    activation.context.gc_context,
                )?,
                name: translation_unit.pool_string_option(name.0, activation.context.gc_context)?,
                params: Vec::new(),
            },
            AbcMultiname::MultinameL { .. } | AbcMultiname::MultinameLA { .. } => {
                let name = activation.avm2().pop();
                Self::from_multiname_late(translation_unit, abc_multiname, name, activation)?
            }
            AbcMultiname::TypeName { .. } => {
                return Err("Recursive TypeNames are not supported!".into())
            }
        })
    }

    /// Retrieve a given multiname index from the ABC file, yielding an error
    /// if the multiname index is zero.
    pub fn resolve_multiname_index(
        abc: &AbcFile,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<&AbcMultiname, Error> {
        let actual_index: Result<usize, Error> = (multiname_index.0 as usize)
            .checked_sub(1)
            .ok_or_else(|| "Attempted to resolve a multiname at index zero. This is a bug.".into());
        let actual_index = actual_index?;
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        abc_multiname
    }

    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    pub fn from_abc_multiname(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let abc = translation_unit.abc();
        let abc_multiname = Self::resolve_multiname_index(&abc, multiname_index)?;

        match abc_multiname {
            AbcMultiname::TypeName {
                base_type,
                parameters,
            } => {
                let base_multiname = Self::resolve_multiname_index(&abc, *base_type)?;
                let mut base =
                    Self::resolve_multiname_params(translation_unit, base_multiname, activation)?;

                if parameters.len() > 1 {
                    return Err(format!(
                        "VerifyError: Multiname has {} parameters, no more than 1 is allowed",
                        parameters.len()
                    )
                    .into());
                }

                for param_type in parameters {
                    let param_multiname =
                        Self::from_abc_multiname(translation_unit, *param_type, activation)?;

                    base.params.push(param_multiname);
                }

                Ok(base)
            }
            abc_multiname => {
                Self::resolve_multiname_params(translation_unit, abc_multiname, activation)
            }
        }
    }

    /// Read a static multiname from the ABC constant pool
    ///
    /// This function prohibits the use of runtime-qualified and late-bound
    /// names. Runtime multinames will instead result in an error.
    ///
    /// Multiname index zero is also treated as an error, you must check for it
    /// and substitute it with whatever default is called for by AVM2.
    pub fn from_abc_multiname_static(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index: Result<usize, Error> =
            (multiname_index.0 as usize).checked_sub(1).ok_or_else(|| {
                "Attempted to resolve a (static) multiname at index zero. This is a bug.".into()
            });
        let actual_index = actual_index?;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(
                        translation_unit,
                        *namespace,
                        mc,
                    )?],
                    name: translation_unit.pool_string_option(name.0, mc)?,
                    params: Vec::new(),
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(translation_unit, *namespace_set, mc)?,
                name: translation_unit.pool_string_option(name.0, mc)?,
                params: Vec::new(),
            },
            AbcMultiname::TypeName {
                base_type,
                parameters,
            } => {
                let mut base = Self::from_abc_multiname_static(translation_unit, *base_type, mc)?;

                if parameters.len() > 1 {
                    return Err(format!(
                        "VerifyError: Multiname has {} parameters, no more than 1 is allowed",
                        parameters.len()
                    )
                    .into());
                }

                for param_type in parameters {
                    let param_multiname = if param_type.0 == 0 {
                        Self::any()
                    } else {
                        Self::from_abc_multiname_static(translation_unit, *param_type, mc)?
                    };

                    base.params.push(param_multiname);
                }

                base
            }
            _ => return Err(format!("Multiname {} is not static", multiname_index.0).into()),
        })
    }

    /// Indicates the any type (any name in any namespace).
    pub fn any() -> Self {
        Self {
            ns: vec![Namespace::Any],
            name: None,
            params: Vec::new(),
        }
    }

    pub fn public(name: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns: vec![Namespace::public()],
            name: Some(name.into()),
            params: Vec::new(),
        }
    }

    pub fn namespace_set(&self) -> impl Iterator<Item = &Namespace<'gc>> {
        self.ns.iter()
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.name
    }

    pub fn contains_public_namespace(&self) -> bool {
        self.ns.iter().any(|ns| ns.is_public())
    }

    /// Indicates if this multiname matches any type in any namespace.
    pub fn is_any(&self) -> bool {
        self.ns.contains(&Namespace::Any) && self.name.is_none()
    }

    /// Determine if this multiname matches a given QName.
    pub fn contains_name(&self, name: &QName<'gc>) -> bool {
        let ns_match = self
            .ns
            .iter()
            .any(|ns| *ns == Namespace::Any || *ns == name.namespace());
        let name_match = self.name.map(|n| n == name.local_name()).unwrap_or(true);

        ns_match && name_match
    }

    /// List the parameters that the selected class must match.
    pub fn params(&self) -> &[Multiname<'gc>] {
        &self.params[..]
    }
}

impl<'gc> From<QName<'gc>> for Multiname<'gc> {
    fn from(q: QName<'gc>) -> Self {
        Self {
            ns: vec![q.ns],
            name: Some(q.name),
            params: Vec::new(),
        }
    }
}

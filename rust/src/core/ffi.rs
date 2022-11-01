use std::ffi::{c_void, CStr};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;
use std::{fmt, ptr};

use opendp_derive::bootstrap;

use crate::error::{Error, ErrorVariant, ExplainUnwrap, Fallible};
use crate::ffi::any::{
    AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyTransformation,
    IntoAnyFunctionExt, IntoAnyMeasurementExt, IntoAnyTransformationExt,
};
use crate::ffi::util::into_c_char_p;
use crate::ffi::util::{self, c_bool};
use crate::{try_, try_as_ref};

#[repr(C)]
pub struct FfiSlice {
    pub ptr: *const c_void,
    pub len: usize,
}

impl FfiSlice {
    pub fn new(ptr: *mut c_void, len: usize) -> Self {
        Self { ptr, len }
    }
}

#[repr(C)]
pub struct FfiError {
    pub variant: *mut c_char,
    pub message: *mut c_char,
    // MAY BE NULL!
    pub backtrace: *mut c_char,
}

impl FfiError {
    fn variant_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.variant)
                .to_str()
                .unwrap_or("Couldn't get variant!")
        }
    }

    fn message_str(&self) -> Option<&str> {
        unsafe {
            self.message.as_ref().map(|s| {
                CStr::from_ptr(s)
                    .to_str()
                    .unwrap_or("Couldn't get message!")
            })
        }
    }
}

impl From<Error> for FfiError {
    fn from(mut error: Error) -> Self {
        Self {
            variant: try_!(util::into_c_char_p(format!("{:?}", error.variant))),
            message: try_!(error.message.map_or(
                Ok(ptr::null::<c_char>() as *mut c_char),
                util::into_c_char_p
            )),
            backtrace: try_!(util::into_c_char_p(
                if let ErrorVariant::RelationDebug = error.variant {
                    String::default()
                } else {
                    error.backtrace.resolve();
                    format!("{:?}", error.backtrace)
                }
            )),
        }
    }
}

impl Drop for FfiError {
    fn drop(&mut self) {
        let _variant =
            util::into_string(self.variant).unwrap_assert("variants do not contain null bytes");
        let _message = unsafe { self.message.as_mut() }.map(|p| util::into_string(p).unwrap());
        let _backtrace = util::into_string(self.backtrace).unwrap();
    }
}

impl PartialEq for FfiError {
    fn eq(&self, other: &Self) -> bool {
        self.variant_str() == other.variant_str() && self.message_str() == other.message_str()
    }
}

impl Debug for FfiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FfiError: {{ type: {}, message: {:?} }}",
            self.variant_str(),
            self.message_str()
        )
    }
}

// Using this repr means we'll get a tagged union in C.
// Because this is a generic, we need to be careful about sizes. Currently, everything that goes in here
// is a pointer, so we're OK, but we may need to revisit this.
#[repr(C, u32)]
pub enum FfiResult<T> {
    Ok(T),
    Err(*mut FfiError),
}

impl<TI, TO: From<TI>> From<Fallible<TI>> for FfiResult<*mut TO> {
    fn from(result: Fallible<TI>) -> Self {
        result.map_or_else(
            |e| Self::Err(util::into_raw(FfiError::from(e))),
            |v| Self::Ok(util::into_raw(TO::from(v))),
        )
    }
}

impl<T> From<Error> for FfiResult<T> {
    fn from(e: Error) -> Self {
        Self::Err(util::into_raw(FfiError::from(e)))
    }
}

impl<T: PartialEq> PartialEq for FfiResult<*mut T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ok(self_), Self::Ok(other)) => util::as_ref(*self_) == util::as_ref(*other),
            (Self::Err(self_), Self::Err(other)) => util::as_ref(*self_) == util::as_ref(*other),
            _ => false,
        }
    }
}

impl<T: Debug> Debug for FfiResult<*mut T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FfiResult::Ok(ok) => write!(f, "Ok({:?})", util::as_ref(*ok).unwrap_test()),
            FfiResult::Err(err) => write!(f, "Err({:?})", util::as_ref(*err).unwrap_test()),
        }
    }
}

/// Trait to convert Result<Measurement> into FfiResult<*mut AnyMeasurement>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyMeasurementFfiResultExt {
    fn into_any(self) -> FfiResult<*mut AnyMeasurement>;
}

impl<T: IntoAnyMeasurementExt> IntoAnyMeasurementFfiResultExt for Fallible<T> {
    fn into_any(self) -> FfiResult<*mut AnyMeasurement> {
        self.map(IntoAnyMeasurementExt::into_any).into()
    }
}

/// Trait to convert Result<Transformation> into FfiResult<*mut AnyTransformation>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyTransformationFfiResultExt {
    fn into_any(self) -> FfiResult<*mut AnyTransformation>;
}

impl<T: IntoAnyTransformationExt> IntoAnyTransformationFfiResultExt for Fallible<T> {
    fn into_any(self) -> FfiResult<*mut AnyTransformation> {
        self.map(IntoAnyTransformationExt::into_any).into()
    }
}

pub trait IntoAnyFunctionFfiResultExt {
    fn into_any(self) -> FfiResult<*mut AnyFunction>;
}

impl<T: IntoAnyFunctionExt> IntoAnyFunctionFfiResultExt for Fallible<T> {
    fn into_any(self) -> FfiResult<*mut AnyFunction> {
        self.map(IntoAnyFunctionExt::into_any).into()
    }
}

impl From<FfiError> for Error {
    fn from(val: FfiError) -> Self {
        let variant = util::to_str(val.variant).unwrap_assert("variants do not contain null bytes");
        let variant = match variant {
            "FFI" => ErrorVariant::FFI,
            "TypeParse" => ErrorVariant::TypeParse,
            "FailedFunction" => ErrorVariant::FailedFunction,
            "FailedRelation" => ErrorVariant::FailedRelation,
            "RelationDebug" => ErrorVariant::RelationDebug,
            "FailedCast" => ErrorVariant::FailedCast,
            "DomainMismatch" => ErrorVariant::DomainMismatch,
            "MakeTransformation" => ErrorVariant::MakeTransformation,
            "MakeMeasurement" => ErrorVariant::MakeMeasurement,
            "InvalidDistance" => ErrorVariant::InvalidDistance,
            "NotImplemented" => ErrorVariant::NotImplemented,
            unknown => return err!(NotImplemented, "Unknown ErrorVariant {}", unknown),
        };
        Error {
            variant,
            message: util::to_option_str(val.message)
                .unwrap_test()
                .map(|s| s.to_owned()),
            backtrace: backtrace::Backtrace::new_unresolved(),
        }
    }
}

impl<T> From<FfiResult<*mut T>> for Fallible<T> {
    fn from(result: FfiResult<*mut T>) -> Self {
        match result {
            FfiResult::Ok(ok) => Ok(util::into_owned(ok)?),
            FfiResult::Err(err) => Err(util::into_owned(err)?.into()),
        }
    }
}

#[bootstrap(
    name = "_error_free",
    arguments(this(c_type = "FfiError *", do_not_convert = true, hint = "FfiError"))
)]
/// Internal function. Free the memory associated with `error`.
///
/// # Returns
/// A boolean, where true indicates successful free
#[no_mangle]
#[must_use]
pub extern "C" fn opendp_core___error_free(this: *mut FfiError) -> bool {
    util::into_owned(this).is_ok()
}

#[bootstrap(
    name = "transformation_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the input domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_domain(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "transformation_output_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the output domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_domain(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_domain.clone()))
}

#[bootstrap(
    name = "transformation_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the input domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_metric(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "transformation_output_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the output domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_metric(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_metric.clone()))
}

#[bootstrap(
    name = "measurement_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the input domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_domain(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "measurement_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the input domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_metric(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "measurement_output_measure",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>", do_not_convert = true)
)]
/// Get the output domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_output_measure(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyMeasure> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_measure.clone()))
}

#[bootstrap(
    name = "measurement_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a measurement.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_function(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "transformation_map",
    arguments(
        transformation(rust_type = b"null"),
        distance_in(rust_type = "$transformation_input_distance_type(transformation)")
    )
)]
/// Use the `transformation` to map a given `d_in` to `d_out`.
///
/// # Arguments
/// * `transformation` - Transformation to check the map distances with.
/// * `distance_in` - Distance in terms of the input metric.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_map(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = transformation.map(distance_in);
    distance_out.into()
}

#[bootstrap(
    name = "transformation_check",
    arguments(
        transformation(rust_type = b"null"),
        distance_in(rust_type = "$transformation_input_distance_type(transformation)"),
        distance_out(rust_type = "$transformation_output_distance_type(transformation)"),
    ),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
///
/// # Arguments
/// * `measurement` - Measurement to check the privacy relation of.
/// * `d_in` - Distance in terms of the input metric.
/// * `d_out` - Distance in terms of the output metric.
///
/// # Returns
/// True indicates that the relation passed at the given distance.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_check(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(transformation.check(distance_in, distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "measurement_map",
    arguments(
        measurement(rust_type = b"null"),
        distance_in(rust_type = "$measurement_input_distance_type(measurement)"),
        distance_out(rust_type = "$measurement_output_distance_type(measurement)"),
    )
)]
/// Use the `measurement` to map a given `d_in` to `d_out`.
///
/// # Arguments
/// * `measurement` - Measurement to check the map distances with.
/// * `distance_in` - Distance in terms of the input metric.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_map(
    measurement: *const AnyMeasurement,
    distance_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let measurement = try_as_ref!(measurement);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = measurement.map(distance_in);
    distance_out.into()
}

#[bootstrap(
    name = "measurement_check",
    arguments(
        measurement(rust_type = b"null"),
        distance_in(rust_type = "$measurement_input_distance_type(measurement)"),
        distance_out(rust_type = "$measurement_output_distance_type(measurement)"),
    ),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
///
/// # Arguments
/// * `measurement` - Measurement to check the privacy relation of.
/// * `d_in` - Distance in terms of the input metric.
/// * `d_out` - Distance in terms of the output metric.
///
/// # Returns
/// True indicates that the relation passed at the given distance.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_check(
    measurement: *const AnyMeasurement,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let measurement = try_as_ref!(measurement);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(measurement.check(distance_in, distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "measurement_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$measurement_input_carrier_type(this)")
    )
)]
/// Invoke the `measurement` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Measurement to invoke.
/// * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_invoke(
    this: *const AnyMeasurement,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "_measurement_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___measurement_free(this: *mut AnyMeasurement) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "transformation_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$transformation_input_carrier_type(this)")
    )
)]
/// Invoke the `transformation` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Transformation to invoke.
/// * `arg` - Input data to supply to the transformation. A member of the transformation's input domain.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_invoke(
    this: *const AnyTransformation,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "transformation_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a transformation.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_function(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "_transformation_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___transformation_free(
    this: *mut AnyTransformation,
) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "transformation_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_carrier_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_carrier_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "transformation_input_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input distance type of `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_distance_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "transformation_output_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the output distance type of `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_distance_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.output_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_input_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input distance type of `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_distance_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_output_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the output distance type of `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_output_distance_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.output_measure.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "function_eval",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$parse_or_infer(TI, arg)"),
        TI(rust_type = b"null", default = b"null"),
    )
)]
/// Eval the `function` with `arg`.
///
/// # Arguments
/// * `this` - Function to invoke.
/// * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
/// * `TI` - Input Type.
#[no_mangle]
pub extern "C" fn opendp_core__function_eval(
    this: *const AnyFunction,
    arg: *const AnyObject,
    TI: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    let _TI = TI;
    this.eval(arg).into()
}

#[bootstrap(
    name = "_function_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___function_free(this: *mut AnyFunction) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[cfg(test)]
mod tests {
    use crate::combinators::tests::{make_test_measurement, make_test_transformation};
    use crate::ffi::any::{Downcast, IntoAnyMeasurementExt, IntoAnyTransformationExt};
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_ffi_error_from_error() {
        let err = err!(FailedFunction, "Eat my shorts!");
        let ffi_err: FfiError = err.into();
        assert_eq!(
            ffi_err,
            FfiError {
                variant: "FailedFunction".to_char_p(),
                message: "Eat my shorts!".to_char_p(),
                backtrace: "".to_char_p(),
            }
        )
    }

    #[test]
    fn test_ffi_result_from_result_ok() {
        let res = Ok(999);
        let ffi_res = FfiResult::from(res);
        assert_eq!(FfiResult::Ok(util::into_raw(999)), ffi_res);
    }

    #[test]
    fn test_ffi_result_from_result_err() {
        let res: Fallible<i32> = fallible!(FailedFunction, "Eat my shorts!");
        let ffi_res: FfiResult<*mut i32> = FfiResult::from(res);
        assert_eq!(
            ffi_res,
            FfiResult::Err(util::into_raw(FfiError {
                variant: "FailedFunction".to_char_p(),
                message: "Eat my shorts!".to_char_p(),
                backtrace: "".to_char_p(),
            }))
        );
    }

    #[test]
    fn test_error_from_ffi_error() {
        let ffi_err = FfiError {
            variant: "FailedFunction".to_char_p(),
            message: "Eat my shorts!".to_char_p(),
            backtrace: "".to_char_p(),
        };
        let err: Error = ffi_err.into();
        assert_eq!(err, err!(FailedFunction, "Eat my shorts!"))
    }

    #[test]
    fn test_result_from_ffi_result_ok() {
        let ffi_res = FfiResult::Ok(util::into_raw(123));
        let res = Fallible::from(ffi_res);
        assert_eq!(res, Ok(123));
    }

    #[test]
    fn test_result_from_ffi_result_err() {
        let ffi_res: FfiResult<*mut i32> = FfiResult::Err(util::into_raw(FfiError {
            variant: "FailedFunction".to_char_p(),
            message: "Eat my shorts!".to_char_p(),
            backtrace: "".to_char_p(),
        }));
        let res = Fallible::from(ffi_res);
        assert_eq!(res, fallible!(FailedFunction, "Eat my shorts!"));
    }

    #[test]
    fn test_measurement_invoke() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>().into_any());
        let arg = AnyObject::new_raw(vec![999]);
        let res = opendp_core__measurement_invoke(measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_measurement_invoke_wrong_type() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>().into_any());
        let arg = AnyObject::new_raw(vec![999.0]);
        let res = Fallible::from(opendp_core__measurement_invoke(measurement, arg));
        assert_eq!(res.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>().into_any());
        let arg = AnyObject::new_raw(vec![999]);
        let res = opendp_core__transformation_invoke(transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![999]);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke_wrong_type() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>().into_any());
        let arg = AnyObject::new_raw(999.0);
        let res = Fallible::from(opendp_core__transformation_invoke(transformation, arg));
        assert_eq!(res.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}

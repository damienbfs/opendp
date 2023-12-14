# Auto-generated. Do not edit!
'''
The ``measures`` modules provides functions that measure the distance between probability distributions.
'''
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
    "_measure_free",
    "fixed_smoothed_max_divergence",
    "max_divergence",
    "measure_debug",
    "measure_distance_type",
    "measure_type",
    "smoothed_max_divergence",
    "user_divergence",
    "zero_concentrated_divergence"
]


@versioned
def _measure_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.
    
    [_measure_free in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn._measure_free.html)
    
    :param this: 
    :type this: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this
    
    # Call library function.
    lib_function = lib.opendp_measures___measure_free
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    
    return output


@versioned
def fixed_smoothed_max_divergence(
    T: RuntimeTypeDescriptor
) -> Measure:
    r"""Construct an instance of the `FixedSmoothedMaxDivergence` measure.
    
    [fixed_smoothed_max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.fixed_smoothed_max_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__fixed_smoothed_max_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output


@versioned
def max_divergence(
    T: RuntimeTypeDescriptor
) -> Measure:
    r"""Construct an instance of the `MaxDivergence` measure.
    
    [max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.max_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__max_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output


@versioned
def measure_debug(
    this: Measure
) -> str:
    r"""Debug a `measure`.
    
    [measure_debug in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.measure_debug.html)
    
    :param this: The measure to debug (stringify).
    :type this: Measure
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measures__measure_debug
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def measure_distance_type(
    this: Measure
) -> str:
    r"""Get the distance type of a `measure`.
    
    [measure_distance_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.measure_distance_type.html)
    
    :param this: The measure to retrieve the distance type from.
    :type this: Measure
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measures__measure_distance_type
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def measure_type(
    this: Measure
) -> str:
    r"""Get the type of a `measure`.
    
    [measure_type in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.measure_type.html)
    
    :param this: The measure to retrieve the type from.
    :type this: Measure
    :rtype: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)
    
    # Call library function.
    lib_function = lib.opendp_measures__measure_type
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    
    return output


@versioned
def smoothed_max_divergence(
    T: RuntimeTypeDescriptor
) -> Measure:
    r"""Construct an instance of the `SmoothedMaxDivergence` measure.
    
    [smoothed_max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.smoothed_max_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__smoothed_max_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output


@versioned
def user_divergence(
    descriptor: str
) -> Measure:
    r"""Construct a new UserDivergence.
    Any two instances of an UserDivergence are equal if their string descriptors are equal.
    
    [user_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.user_divergence.html)
    
    :param descriptor: A string description of the privacy measure.
    :type descriptor: str
    :rtype: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("honest-but-curious")
    
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_descriptor = py_to_c(descriptor, c_type=ctypes.c_char_p, type_name=String)
    
    # Call library function.
    lib_function = lib.opendp_measures__user_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_descriptor), Measure))
    
    return output


@versioned
def zero_concentrated_divergence(
    T: RuntimeTypeDescriptor
) -> Measure:
    r"""Construct an instance of the `ZeroConcentratedDivergence` measure.
    
    [zero_concentrated_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.zero_concentrated_divergence.html)
    
    :param T: 
    :type T: :py:ref:`RuntimeTypeDescriptor`
    :rtype: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # Standardize type arguments.
    T = RuntimeType.parse(type_name=T)
    
    # Convert arguments to c types.
    c_T = py_to_c(T, c_type=ctypes.c_char_p)
    
    # Call library function.
    lib_function = lib.opendp_measures__zero_concentrated_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult
    
    output = c_to_py(unwrap(lib_function(c_T), Measure))
    
    return output

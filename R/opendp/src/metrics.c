// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

#include "convert.h"
#include "convert_elements.h"
#include "Ropendp.h"
#include "opendp.h"
#include "opendp_extras.h"


SEXP metrics__absolute_distance(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__absolute_distance(c_T);

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__change_one_distance(
    SEXP log
) {
    // No arguments to convert to c types.
    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__change_one_distance();

    UNPROTECT(0);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__discrete_distance(
    SEXP log
) {
    // No arguments to convert to c types.
    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__discrete_distance();

    UNPROTECT(0);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__hamming_distance(
    SEXP log
) {
    // No arguments to convert to c types.
    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__hamming_distance();

    UNPROTECT(0);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__insert_delete_distance(
    SEXP log
) {
    // No arguments to convert to c types.
    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__insert_delete_distance();

    UNPROTECT(0);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__l1_distance(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__l1_distance(c_T);

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__l2_distance(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__l2_distance(c_T);

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__linf_diff_distance(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__linf_diff_distance(c_T);

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP metrics__metric_debug(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    AnyMetric * c_this = sexp_to_anymetricptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_metrics__metric_debug(c_this);

    UNPROTECT(1);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP metrics__metric_distance_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    AnyMetric * c_this = sexp_to_anymetricptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_metrics__metric_distance_type(c_this);

    UNPROTECT(1);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP metrics__metric_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    AnyMetric * c_this = sexp_to_anymetricptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_metrics__metric_type(c_this);

    UNPROTECT(1);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP metrics__symmetric_distance(
    SEXP log
) {
    // No arguments to convert to c types.
    // Call library function.
    FfiResult_____AnyMetric _result = opendp_metrics__symmetric_distance();

    UNPROTECT(0);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


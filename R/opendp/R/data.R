# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Internal function. Retrieve the type descriptor string of an AnyObject.
#'
#' @concept data
#' @param this A pointer to the AnyObject.
#' @return str
object_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("object_type", "data", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "data__object_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Internal function. Use an SMDCurve to find beta at a given `alpha`.
#'
#' @concept data
#' @param curve The SMDCurve.
#' @param alpha What to fix alpha to compute beta.
#' @return Beta at a given `alpha`.
smd_curve_beta <- function(
  curve,
  alpha
) {
  # No type arguments to standardize.
  log <- new_constructor_log("smd_curve_beta", "data", new_hashtab(
    list("curve", "alpha"),
    list(curve, unbox2(alpha))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "data__smd_curve_beta",
    curve, alpha,
    log, PACKAGE = "opendp")
  output
}


#' Internal function. Use an SMDCurve to find epsilon at a given `delta`.
#'
#' @concept data
#' @param curve The SMDCurve.
#' @param delta What to fix delta to compute epsilon.
#' @return Epsilon at a given `delta`.
smd_curve_epsilon <- function(
  curve,
  delta
) {
  # Standardize type arguments.
  .T.delta <- get_atom(object_type(curve))

  log <- new_constructor_log("smd_curve_epsilon", "data", new_hashtab(
    list("curve", "delta"),
    list(curve, delta)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.delta, inferred = rt_infer(delta))

  # Call wrapper function.
  output <- .Call(
    "data__smd_curve_epsilon",
    curve, delta, rt_parse(.T.delta),
    log, PACKAGE = "opendp")
  output
}


#' Internal function. Use an SMDCurve to construct a piecewise linear supporting function.
#'
#' @concept data
#' @param curve The SMDCurve.
#' @param num_approximations Number of supporting functions to create.
#' @return `α(β)` tradeoff function.
smd_curve_tradeoff <- function(
  curve,
  num_approximations
) {
  # Standardize type arguments.
  .T.num_approximations <- new_runtime_type(origin = "Option", args = list(u32))

  log <- new_constructor_log("smd_curve_tradeoff", "data", new_hashtab(
    list("curve", "num_approximations"),
    list(curve, num_approximations)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.num_approximations, inferred = rt_infer(num_approximations))

  # Call wrapper function.
  output <- .Call(
    "data__smd_curve_tradeoff",
    curve, num_approximations, rt_parse(.T.num_approximations),
    log, PACKAGE = "opendp")
  output
}

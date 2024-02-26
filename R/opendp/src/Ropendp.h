// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

extern SEXP AnyObject_tag;
extern SEXP AnyTransformation_tag;
extern SEXP AnyMeasurement_tag;
extern SEXP AnyDomain_tag;
extern SEXP AnyMetric_tag;
extern SEXP AnyMeasure_tag;
extern SEXP AnyFunction_tag;

SEXP accuracy__accuracy_to_discrete_gaussian_scale(SEXP accuracy, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__accuracy_to_discrete_laplacian_scale(SEXP accuracy, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__accuracy_to_gaussian_scale(SEXP accuracy, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__accuracy_to_laplacian_scale(SEXP accuracy, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__discrete_gaussian_scale_to_accuracy(SEXP scale, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__discrete_laplacian_scale_to_accuracy(SEXP scale, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__gaussian_scale_to_accuracy(SEXP scale, SEXP alpha, SEXP T, SEXP log);
SEXP accuracy__laplacian_scale_to_accuracy(SEXP scale, SEXP alpha, SEXP T, SEXP log);
SEXP combinators__make_basic_composition(SEXP measurements, SEXP T_measurements, SEXP log);
SEXP combinators__make_chain_mt(SEXP measurement1, SEXP transformation0, SEXP log);
SEXP combinators__make_chain_pm(SEXP postprocess1, SEXP measurement0, SEXP log);
SEXP combinators__make_chain_tt(SEXP transformation1, SEXP transformation0, SEXP log);
SEXP combinators__make_fix_delta(SEXP measurement, SEXP delta, SEXP T_delta, SEXP log);
SEXP combinators__make_population_amplification(SEXP measurement, SEXP population_size, SEXP log);
SEXP combinators__make_pureDP_to_fixed_approxDP(SEXP measurement, SEXP log);
SEXP combinators__make_pureDP_to_zCDP(SEXP measurement, SEXP log);
SEXP combinators__make_sequential_composition(SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP d_in, SEXP d_mids, SEXP QO, SEXP T_d_in, SEXP T_d_mids, SEXP log);
SEXP combinators__make_zCDP_to_approxDP(SEXP measurement, SEXP log);
SEXP core__function_eval(SEXP this, SEXP arg, SEXP TI, SEXP T_arg, SEXP log);
SEXP core__measurement_check(SEXP measurement, SEXP distance_in, SEXP distance_out, SEXP T_distance_in, SEXP T_distance_out, SEXP log);
SEXP core__measurement_function(SEXP this, SEXP log);
SEXP core__measurement_input_carrier_type(SEXP this, SEXP log);
SEXP core__measurement_input_distance_type(SEXP this, SEXP log);
SEXP core__measurement_input_domain(SEXP this, SEXP log);
SEXP core__measurement_input_metric(SEXP this, SEXP log);
SEXP core__measurement_invoke(SEXP this, SEXP arg, SEXP T_arg, SEXP log);
SEXP core__measurement_map(SEXP measurement, SEXP distance_in, SEXP T_distance_in, SEXP log);
SEXP core__measurement_output_distance_type(SEXP this, SEXP log);
SEXP core__measurement_output_measure(SEXP this, SEXP log);
SEXP core__queryable_eval(SEXP queryable, SEXP query, SEXP T_query, SEXP log);
SEXP core__queryable_query_type(SEXP this, SEXP log);
SEXP core__transformation_check(SEXP transformation, SEXP distance_in, SEXP distance_out, SEXP T_distance_in, SEXP T_distance_out, SEXP log);
SEXP core__transformation_function(SEXP this, SEXP log);
SEXP core__transformation_input_carrier_type(SEXP this, SEXP log);
SEXP core__transformation_input_distance_type(SEXP this, SEXP log);
SEXP core__transformation_input_domain(SEXP this, SEXP log);
SEXP core__transformation_input_metric(SEXP this, SEXP log);
SEXP core__transformation_invoke(SEXP this, SEXP arg, SEXP T_arg, SEXP log);
SEXP core__transformation_map(SEXP transformation, SEXP distance_in, SEXP T_distance_in, SEXP log);
SEXP core__transformation_output_distance_type(SEXP this, SEXP log);
SEXP core__transformation_output_domain(SEXP this, SEXP log);
SEXP core__transformation_output_metric(SEXP this, SEXP log);
SEXP data__object_type(SEXP this, SEXP log);
SEXP data__smd_curve_epsilon(SEXP curve, SEXP delta, SEXP T_delta, SEXP log);
SEXP data__to_string(SEXP this, SEXP log);
SEXP domains__atom_domain(SEXP bounds, SEXP nullable, SEXP T, SEXP T_bounds, SEXP log);
SEXP domains__domain_carrier_type(SEXP this, SEXP log);
SEXP domains__domain_debug(SEXP this, SEXP log);
SEXP domains__domain_type(SEXP this, SEXP log);
SEXP domains__map_domain(SEXP key_domain, SEXP value_domain, SEXP log);
SEXP domains__member(SEXP this, SEXP val, SEXP T_val, SEXP log);
SEXP domains__option_domain(SEXP element_domain, SEXP D, SEXP log);
SEXP domains__vector_domain(SEXP atom_domain, SEXP size, SEXP T_size, SEXP log);
SEXP measurements__make_alp_queryable(SEXP input_domain, SEXP input_metric, SEXP scale, SEXP total_limit, SEXP value_limit, SEXP size_factor, SEXP alpha, SEXP CO, SEXP CI, SEXP T_value_limit, SEXP T_size_factor, SEXP T_alpha, SEXP log);
SEXP measurements__make_gaussian(SEXP input_domain, SEXP input_metric, SEXP scale, SEXP k, SEXP MO, SEXP QO, SEXP T_k, SEXP log);
SEXP measurements__make_geometric(SEXP input_domain, SEXP input_metric, SEXP scale, SEXP bounds, SEXP QO, SEXP T, SEXP OptionT, SEXP log);
SEXP measurements__make_laplace(SEXP input_domain, SEXP input_metric, SEXP scale, SEXP k, SEXP QO, SEXP T_scale, SEXP T_k, SEXP log);
SEXP measurements__make_laplace_threshold(SEXP input_domain, SEXP input_metric, SEXP scale, SEXP threshold, SEXP k, SEXP TV, SEXP log);
SEXP measurements__make_randomized_response(SEXP categories, SEXP prob, SEXP T, SEXP QO, SEXP T_categories, SEXP log);
SEXP measurements__make_randomized_response_bool(SEXP prob, SEXP constant_time, SEXP QO, SEXP log);
SEXP measurements__make_report_noisy_max_gumbel(SEXP input_domain, SEXP input_metric, SEXP scale, SEXP optimize, SEXP QO, SEXP log);
SEXP measures__fixed_smoothed_max_divergence(SEXP T, SEXP log);
SEXP measures__max_divergence(SEXP T, SEXP log);
SEXP measures__measure_debug(SEXP this, SEXP log);
SEXP measures__measure_distance_type(SEXP this, SEXP log);
SEXP measures__measure_type(SEXP this, SEXP log);
SEXP measures__smoothed_max_divergence(SEXP T, SEXP log);
SEXP measures__user_divergence(SEXP descriptor, SEXP log);
SEXP measures__zero_concentrated_divergence(SEXP T, SEXP log);
SEXP metrics__absolute_distance(SEXP T, SEXP log);
SEXP metrics__change_one_distance(SEXP log);
SEXP metrics__discrete_distance(SEXP log);
SEXP metrics__hamming_distance(SEXP log);
SEXP metrics__insert_delete_distance(SEXP log);
SEXP metrics__l1_distance(SEXP T, SEXP log);
SEXP metrics__l2_distance(SEXP T, SEXP log);
SEXP metrics__linf_distance(SEXP monotonic, SEXP T, SEXP log);
SEXP metrics__metric_debug(SEXP this, SEXP log);
SEXP metrics__metric_distance_type(SEXP this, SEXP log);
SEXP metrics__metric_type(SEXP this, SEXP log);
SEXP metrics__partition_distance(SEXP metric, SEXP log);
SEXP metrics__symmetric_distance(SEXP log);
SEXP metrics__user_distance(SEXP descriptor, SEXP log);
SEXP transformations__choose_branching_factor(SEXP size_guess, SEXP log);
SEXP transformations__make_b_ary_tree(SEXP input_domain, SEXP input_metric, SEXP leaf_count, SEXP branching_factor, SEXP log);
SEXP transformations__make_bounded_float_checked_sum(SEXP size_limit, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_bounded_float_ordered_sum(SEXP size_limit, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_bounded_int_monotonic_sum(SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_bounded_int_ordered_sum(SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_bounded_int_split_sum(SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_cast(SEXP input_domain, SEXP input_metric, SEXP TOA, SEXP log);
SEXP transformations__make_cast_default(SEXP input_domain, SEXP input_metric, SEXP TOA, SEXP TIA, SEXP M, SEXP log);
SEXP transformations__make_cast_inherent(SEXP input_domain, SEXP input_metric, SEXP TOA, SEXP log);
SEXP transformations__make_cdf(SEXP TA, SEXP log);
SEXP transformations__make_clamp(SEXP input_domain, SEXP input_metric, SEXP bounds, SEXP TA, SEXP T_bounds, SEXP log);
SEXP transformations__make_consistent_b_ary_tree(SEXP branching_factor, SEXP TIA, SEXP TOA, SEXP log);
SEXP transformations__make_count(SEXP input_domain, SEXP input_metric, SEXP TO, SEXP log);
SEXP transformations__make_count_by(SEXP input_domain, SEXP input_metric, SEXP MO, SEXP TV, SEXP log);
SEXP transformations__make_count_by_categories(SEXP input_domain, SEXP input_metric, SEXP categories, SEXP null_category, SEXP MO, SEXP TOA, SEXP TIA, SEXP T_categories, SEXP log);
SEXP transformations__make_count_distinct(SEXP input_domain, SEXP input_metric, SEXP TO, SEXP log);
SEXP transformations__make_create_dataframe(SEXP col_names, SEXP K, SEXP T_col_names, SEXP log);
SEXP transformations__make_df_cast_default(SEXP input_domain, SEXP input_metric, SEXP column_name, SEXP TIA, SEXP TOA, SEXP TK, SEXP M, SEXP log);
SEXP transformations__make_df_is_equal(SEXP input_domain, SEXP input_metric, SEXP column_name, SEXP value, SEXP TIA, SEXP TK, SEXP M, SEXP log);
SEXP transformations__make_drop_null(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_find(SEXP input_domain, SEXP input_metric, SEXP categories, SEXP TIA, SEXP T_categories, SEXP log);
SEXP transformations__make_find_bin(SEXP input_domain, SEXP input_metric, SEXP edges, SEXP TIA, SEXP T_edges, SEXP log);
SEXP transformations__make_identity(SEXP domain, SEXP metric, SEXP log);
SEXP transformations__make_impute_constant(SEXP input_domain, SEXP input_metric, SEXP constant, SEXP T_constant, SEXP log);
SEXP transformations__make_impute_uniform_float(SEXP input_domain, SEXP input_metric, SEXP bounds, SEXP TA, SEXP T_bounds, SEXP log);
SEXP transformations__make_index(SEXP input_domain, SEXP input_metric, SEXP categories, SEXP null, SEXP TOA, SEXP T_categories, SEXP log);
SEXP transformations__make_is_equal(SEXP input_domain, SEXP input_metric, SEXP value, SEXP TIA, SEXP M, SEXP log);
SEXP transformations__make_is_null(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_lipschitz_float_mul(SEXP constant, SEXP bounds, SEXP D, SEXP M, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_mean(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_metric_bounded(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_metric_unbounded(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_ordered_random(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_quantile_score_candidates(SEXP input_domain, SEXP input_metric, SEXP candidates, SEXP alpha, SEXP TIA, SEXP T_candidates, SEXP log);
SEXP transformations__make_quantiles_from_counts(SEXP bin_edges, SEXP alphas, SEXP interpolation, SEXP TA, SEXP F, SEXP T_bin_edges, SEXP T_alphas, SEXP log);
SEXP transformations__make_resize(SEXP input_domain, SEXP input_metric, SEXP size, SEXP constant, SEXP MO, SEXP T_constant, SEXP log);
SEXP transformations__make_select_column(SEXP key, SEXP K, SEXP TOA, SEXP log);
SEXP transformations__make_sized_bounded_float_checked_sum(SEXP size, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_sized_bounded_float_ordered_sum(SEXP size, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_sized_bounded_int_checked_sum(SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_sized_bounded_int_monotonic_sum(SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_sized_bounded_int_ordered_sum(SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_sized_bounded_int_split_sum(SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log);
SEXP transformations__make_split_dataframe(SEXP separator, SEXP col_names, SEXP K, SEXP T_col_names, SEXP log);
SEXP transformations__make_split_lines(SEXP log);
SEXP transformations__make_split_records(SEXP separator, SEXP log);
SEXP transformations__make_subset_by(SEXP indicator_column, SEXP keep_columns, SEXP TK, SEXP T_keep_columns, SEXP log);
SEXP transformations__make_sum(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_sum_of_squared_deviations(SEXP input_domain, SEXP input_metric, SEXP S, SEXP T, SEXP log);
SEXP transformations__make_unordered(SEXP input_domain, SEXP input_metric, SEXP log);
SEXP transformations__make_variance(SEXP input_domain, SEXP input_metric, SEXP ddof, SEXP S, SEXP T, SEXP log);

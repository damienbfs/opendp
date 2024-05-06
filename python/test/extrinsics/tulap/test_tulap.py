import opendp.prelude as dp
import numpy as np  # type: ignore[import]
from opendp._extrinsics.tulap import (
    _ptulap,
    ump_test,
    oneside_pvalue,
)


def test_positive_input():
    """Test with a positive t, checks basic operation"""
    t = np.array([1])  # Adjusted to array
    result = _ptulap(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert result[0] > 0, "Result should be positive for positive t"


def test_negative_input():
    """Test with a negative t, checks basic operation"""
    t = -1
    result = _ptulap(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert result < 1, "Result should be less than 1 for negative t"


def test_array_input():
    """Test with an array of t values"""
    t = np.array([0, 1, -1])
    result = _ptulap(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert len(result) == 3, "Result should have the same length as input"


def test_inf_handling():
    """Test to ensure infinities are handled correctly"""
    result = _ptulap(np.array([np.inf]), epsilon=0.1, delta=0.1)
    assert not np.isinf(result).any(), "Result should not contain infinities"


def test_left_tail_basic():
    """Test the left tail functionality with basic inputs."""
    size = 10
    theta = 0.5
    alpha = 0.05
    epsilon = 0.1
    delta = 0.01
    tail = "left"
    data = True

    ump_test_func = ump_test(theta, size, alpha, epsilon, delta, tail)
    result = ump_test_func(data)
    assert isinstance(result, list), "Result should be a list"
    assert all(
        isinstance(item, float) for item in result
    ), "All items in the result should be floats"


def test_right_tail_single_value(mock_new_function, mock_ptulap):
    # Adjust the mock to return an array that has the correct shape (11 elements for size=10)
    size = 10
    mock_ptulap.return_value = np.array(
        [0.5] * (size + 1)
    )  # Now correctly sized to match the binom.pmf array
    mock_new_function.side_effect = lambda func, TO: func

    theta = 0.5
    epsilon = 0.1
    delta = 0.01
    tail = "right"
    Z = np.array([5, 1])

    # Execute the function
    pvalue_func = oneside_pvalue(theta, size, epsilon, delta, tail)
    pvalue = pvalue_func(Z)

    # Assert that the p-value calculation behaves as expected
    # The dot operation should now succeed without a shape mismatch error
    assert np.all(pvalue >= 0) and np.all(
        pvalue <= 1
    ), "P-values should be within [0, 1]"


def test_ptulap():
    from opendp._extrinsics.tulap import _ptulap

    dp.enable_features("contrib")
    t_values = np.array([0.1, 0.5, 1.0, -0.5, -1.0])  # example input values
    m = 0
    epsilon = 0.5
    delta = 0.3

    ptulap_results = _ptulap(t_values, m, epsilon, delta)
    print(ptulap_results)


def test_ump_test():
    from opendp._extrinsics.tulap import ump_test

    data_for_test = 6
    theta = 0.5  # Probability of success
    size = 10  # Sample size
    alpha = 0.05  # Significance level
    epsilon = 0.1  # Differential privacy parameter epsilon
    delta = 0.01  # Differential privacy parameter delta
    tail = "left"  # Tail of the test
    ump_test_result = ump_test(data_for_test, theta, size, alpha, epsilon, delta, tail)
    print(ump_test_result)


def test_oneside_pvalue():

    from opendp._extrinsics.tulap import oneside_pvalue

    Z_values = np.array([2, 3, 4])  # Example Tulap random variables
    theta = 0.5  # Probability of success in binomial distribution
    size = 10  # Number of trials
    epsilon = 0.5  # Tulap parameter b
    delta = 0.3  # Tulap parameter q
    tail = "right"  # Tail for the p-value calculation

    # Create a function to calculate p-values
    pvalues = oneside_pvalue(Z_values, theta, size, epsilon, delta, tail)
    print(pvalues)
    print("type: ", type(pvalues))


def test_twoside_pvalue():
    from opendp._extrinsics.tulap import twoside_pvalue

    Z_values = np.array([1])  # Example Tulap random variables
    theta = 0.5  # Probability of success in binomial distribution
    size = 10  # Number of trials
    epsilon = 0.5  # Tulap parameter b
    delta = 0.3  # Tulap parameter q

    # Calculate two-sided p-values
    twoside_pvalues = twoside_pvalue(Z_values, theta, size, epsilon, delta)

    print("this is two sided p value: ", twoside_pvalues)
    print("this is two sided p value type:  ", type(twoside_pvalues))


def test_CI_twoside():
    from opendp._extrinsics.tulap import CI_twoside

    # Example parameters for make_CI2
    Z_example = [1]
    alpha = 0.05
    size = 100
    epsilon = 0.1
    delta = 0.01
    tail = "lower"

    # Obtain the DP-wrapped CI function
    result = CI_twoside(Z_example, alpha, size, epsilon, delta, tail)
    print("Result from the DP-wrapped CI function:", result)

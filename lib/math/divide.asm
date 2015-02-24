

; SUBROUTINE: Divide two integers
; ---------------------------------

;     Input: $arg0: dividend as int, $arg1: divisor as int
;    Output: $return's the arg0/arg1 as int division
; Algorithm: TODO

@start(divide, 2)
    math_div_loop:
                                        ; arg0 < arg1 → break
    JLS     :math_div_done      $arg0   $arg1
    ADD     $return             1
    SUB     $arg0               $arg1
    JMP     :math_div_loop              ; Loop iteration

    math_div_done:
@end()
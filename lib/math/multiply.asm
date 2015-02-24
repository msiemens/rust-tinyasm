; Define constants
    $math_mul_counter       = [_]


; SUBROUTINE: Multiply two integers
; ---------------------------------

;     Input: $arg1 & $arg2 as two integers
;    Output: $return's the multiplication of the two
; Algorithm: Sum arg1 arg0' times
@start(multiply, 2)
    math_mul_loop:
                                        ; counter == arg1 → break
    JEQ     :math_mul_done      $arg1   $math_mul_counter
    ADD     $math_mul_counter   1
    ADD     $return             $arg0
    JMP     :math_mul_loop              ; Loop iteration

    math_mul_done:
@end()
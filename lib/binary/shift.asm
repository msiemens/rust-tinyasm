; Define constants
    $shift_r_and        = [_]
    $shift_r_cmp        = [_]
    $shift_r_bit2       = 2
    $shift_r_bit3       = 4
    $shift_r_bit4       = 8
    $shift_r_bit5       = 16
    $shift_r_bit6       = 32
    $shift_r_bit7       = 64
    $shift_r_bit8       = 128


; SUBROUTINE: Shift left
; ----------------------

;  Input: $arg0 as integer
; Output: $return's the integer shifted left
@start(binary_shift_left, 1)
    ADD     $arg0       $arg0
    MOV     $return     $arg0
@end()


; SUBROUTINE: Shift right
; -----------------------

;     Input: $arg0 as integer
;    Output: $return's the integer shifted right
; Algorithm: We check every bit, and if it is set, we add bit_val/2
;            to the result:
;            Input:  100 ← bit(3) is set, value: 4, add 4/2=2
;            Output: 010
@start(binary_shift_right, 1)
    MOV     $return     0                       ; Initialize memory

    ; shift_r_bit2:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit2
    JEQ     :shift_r_bit3   $shift_r_cmp    0   ; v & 2 == 0 → skip
    ADD     $return         1                   ; Add 2 / 2 = 1

    shift_r_bit3:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit3
    JEQ     :shift_r_bit4   $shift_r_cmp    0   ; v & 4 == 0 → skip
    ADD     $return         2                   ; Add 4 / 2 = 2

    shift_r_bit4:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit4
    JEQ     :shift_r_bit5   $shift_r_cmp    0   ; v & 8 == 0 → skip
    ADD     $return         4                   ; Add 8 / 2 = 4

    shift_r_bit5:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit5
    JEQ     :shift_r_bit6   $shift_r_cmp    0   ; v & 16 == 0 → skip
    ADD     $return         8                   ; Add 16 / 2 = 8

    shift_r_bit6:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit6
    JEQ     :shift_r_bit7   $shift_r_cmp    0   ; v & 32 == 0 → skip
    ADD     $return         16                  ; Add 32 / 2 = 16

    shift_r_bit7:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit7
    JEQ     :shift_r_bit8   $shift_r_cmp    0   ; v & 64 == 0 → skip
    ADD     $return         32                  ; Add 64 / 2 = 32

    shift_r_bit8:
    MOV     $shift_r_cmp    $arg0
    AND     $shift_r_cmp    $shift_r_bit8
    JEQ     :shift_r_return $shift_r_cmp    0   ; v & 128 == 0 → skip
    ADD     $return         64                  ; Add 128 / 2 = 64

    shift_r_return:
@end()

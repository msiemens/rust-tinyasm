; Approximate PI
; --------------
;
; by Markus Siemens <markus@es-netze.de>

; Define constants
    $max_rand_square   = 144    ; (RAND_MAX/2) ** 2

    ; Approximate PI
    $pi_iterations   = 100  ; Iteration count
    $pi_rand_divider = 2    ; Divide the RANDOM numbers by this, so we don't overflow
    $pi_counter     = [_]   ; Loop counter
    $pi_rand0       = [_]   ; First RANDOM number
    $pi_rand1       = [_]   ; Second RANDOM number
    $pi_rand_sum    = [_]
    $pi_inside      = [_]   ; Number of dots inside the circle

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

main:
    MOV $pi_counter     0               ; Initialize memory

    main_loop:                          ; The main loop
                                        ; Loop break condition: $pi_counter == $pi_iterations
    JEQ     :print      $pi_counter     $pi_iterations
    APRINT  '.'
    MOV     $pi_rand_sum 0              ; Reset sum of rand0^2 and rand1^2

                                        ; Get random numbers
    RANDOM  $pi_rand0
    @call(divide, $pi_rand0, $pi_rand_divider)
    MOV     $pi_rand0   $return

    RANDOM  $pi_rand1
    @call(divide, $pi_rand1, $pi_rand_divider)
    MOV     $pi_rand1   $return

    @call(multiply, $pi_rand0, $pi_rand0)
    MOV     $pi_rand0   $return

    @call(multiply, $pi_rand1, $pi_rand1)
    MOV     $pi_rand1   $return

    ADD     $pi_rand_sum    $pi_rand0   ; Add $pi_rand0^2 and $pi_rand1^2
    ADD     $pi_rand_sum    $pi_rand1

                                        ; If $pi_rand_sum > $MAX_RAND_SQUARE, GOTO FI
    JGT     :pi_fi_indot    $pi_rand_sum    $max_rand_square
    ADD     $pi_inside      1

    pi_fi_indot:

                                        ; If pi_counter_0 == 255
    ADD     $pi_counter     1
    JMP     :main_loop                  ; Next loop iteration

print:                                  ; SUBROUTINE
                                        ; Calculate PI using 'inside / total * 4' as float
    APRINT '\n'
    DPRINT  $pi_inside
    APRINT  '/'
    DPRINT  $pi_iterations
    APRINT  '*'
    DPRINT  4

    JMP     :end


end:
                                        ; SUBROUTINE
                                        ; End the programm execution
    HALT

#import <lib/math/multiply.asm>
#import <lib/math/divide.asm>
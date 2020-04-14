	.section ".text.boot"

	.global _start
_start:
	ldr x1, =_start
	mov sp, x1

	// Clear bss
	ldr x1, =__bss_start
	ldr w2, =__bss_size
3:
	cbz w2, 4f
	str xzr, [x1], #8
	sub w2, w2, #1
	cbnz w2, 3b
4:
	// C言語プログラムのメイン関数にジャンプ
	bl  entry
	// フェイルセーフの場合は、このコアも停止
	b _start

message:
	.asciz "*** Hello from aarch64! ***"
	.end

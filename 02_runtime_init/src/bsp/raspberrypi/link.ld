/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

/* Raspberryのファームウェアによりカーネルがロードされるアドレス */
__rpi_load_addr = 0x80000;

ENTRY(__rpi_load_addr)

PHDRS
{
    segment_rx PT_LOAD FLAGS(5); /* 5 == RX */
    segment_rw PT_LOAD FLAGS(6); /* 6 == RW */
}

SECTIONS
{
    . =  __rpi_load_addr;
                                        /*   ^             */
                                        /*   | stack       */
                                        /*   | growth      */
                                        /*   | direction   */
   __boot_core_stack_end_exclusive = .; /*   |             */

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text :
    {
        KEEP(*(.text._start))
        *(.text._start_arguments) /* _start()により読み込まれる定数（Rustで言うsttics） */
        *(.text._start_rust)      /* Rustのエントリポイント */
        *(.text*)                 /* その他のすべて */
    } :segment_rx

    .rodata : ALIGN(8) { *(.rodata*) } :segment_rx
    .got    : ALIGN(8) { *(.got)     } :segment_rx

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : { *(.data*) } :segment_rw

    /* セクションはu64のチャンクでゼロ詰めされる。start/endアドレスは8バイトアライン */
    .bss : ALIGN(8)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(8);

        . += 8; /* bss == 0の場合にも __bss_start <= __bss_end_inclusive になるように詰める */
        __bss_end_inclusive = . - 8;
    } :NONE
}

# binutils document

## arm32

```
9.4.2.4 ARM再配置の生成

シンボル名の後に再配置名を括弧で囲むことで、特定のデータ再配置を生成することができます。たとえば、次のようになります。

        .word foo(TARGET1)

これは、シンボル fooに対して、'R_ARM_TARGET1'リロケーションを生成します。次の再配置がサポートされています: GOT, GOTOFF, TARGET1, TARGET2, SBREL, TLSGD, TLSLDM, TLSLDO, TLSDESC, TLSCALL, GOTTPOFF, GOT_PREL, TPOFF.

古いツールチェーンとの互換性のために、アセンブラはブランチターゲットの後に(PLT)も受け付けます。レガシーターゲットでは、非推奨の 'R_ARM_PLT32' リロケーションが生成されます。EABIターゲットでは、必要に応じて'R_ARM_CALL'または'R_ARM_JUMP24'リロケーションがエンコードされます。

'MOVW'および'MOVT'命令のリロケーションは、値の前にそれぞれ'#:lower16:'および'#:upper16'を付けることで生成できます。例えば、fooの32ビットアドレスをr0にロードするには、次のようにします。

        MOVW r0, #:lower16:foo
        MOVT r0, #:upper16:foo

リローケーションの'R_ARM_THM_ALU_ABS_G0_NC', 'R_ARM_THM_ALU_ABS_G1_NC', 'R_ARM_THM_ALU_ABS_G2_NC', 'R_ARM_THM_ALU_ABS_G3_NC'は、値の前に'#: lower0_7:#', '#:lower8_15:#', '#:upper0_7:#', '#:upper8_15:#' をそれぞれ先頭につけることで生成できます。例えば、fooの32ビットアドレスをr0にロードする場合は以下のようにします。

        MOVS r0, #:upper8_15:#foo
        LSLS r0, r0, #8
        ADDS r0, #:upper0_7:#foo
        LSLS r0, r0, #8
        ADDS r0, #:lower8_15:#foo
        LSLS r0, r0, #8
        ADDS r0, #:lower0_7:#foo
```

## aarch64

```
9.1.3.3 再配置

'MOVZ'や'MOVK命令のリローケーションは、ラベルの前に'#:abs_g2:'などを付けることで生成できます。例えば、fooの48ビットの絶対アドレスをx0にロードするには、次のようにします。

        movz x0, #:abs_g2:foo		// bits 32-47, overflow check
        movk x0, #:abs_g1_nc:foo	// bits 16-31, no overflow check
        movk x0, #:abs_g0_nc:foo	// bits  0-15, no overflow check

'ADRP'および'ADD', 'LDR', 'STR'命令のリロケートは、ラベルの前にそれぞれ':pg_hi21:', '#:lo12:'を付けることで生成できます。

例えば、33ビット（+/-4GB）のpc-relative addressingを使用して、fooのアドレスをx0にロードする場合を考えてみましょう。

        adrp x0, :pg_hi21:foo
        add  x0, x0, #:lo12:foo

fooの値をx0にロードする場合は次のとおりです。

        adrp x0, :pg_hi21:foo
        ldr  x0, [x0, #:lo12:foo]

‘:pg_hi21:’はオプションであることに注意してください。

        adrp x0, foo

は、次と同じです。

        adrp x0, :pg_hi21:foo
```

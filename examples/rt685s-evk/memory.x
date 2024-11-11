MEMORY {
	OTFAD    : ORIGIN = 0x08000000, LENGTH = 256
	FCB      : ORIGIN = 0x08000400, LENGTH = 512
	BIV      : ORIGIN = 0x08000600, LENGTH = 4
	KEYSTORE : ORIGIN = 0x08000800, LENGTH = 2K
	FLASH    : ORIGIN = 0x08001000, LENGTH = 1M
	RAM      : ORIGIN = 0x20080000, LENGTH = 1536K
}

# Redirect/rename a function here, so that we can make sure the user has added the linker script to the RUSTFLAGS
EXTERN (__embedded_test_start);
PROVIDE(embedded_test_linker_file_not_added_to_rustflags = __embedded_test_start);

SECTIONS {
	.otfad : {
		. = ALIGN(4);
		KEEP(* (.otfad))
		. = ALIGN(4);
	} > OTFAD

	.fcb : {
		. = ALIGN(4);
		KEEP(* (.fcb))
		. = ALIGN(4);
	} > FCB

	.biv : {
		. = ALIGN(4);
		KEEP(* (.biv))
		. = ALIGN(4);
	} > BIV

	.keystore : {
		. = ALIGN(4);
		KEEP(* (.keystore))
		. = ALIGN(4);
	} > KEYSTORE

	.embedded_test 1 (INFO) :
	{
		KEEP(*(.embedded_test.*));
	}
}

extends TextEdit
signal token_entered(token:PackedByteArray)
signal malformed_token

func parse() -> void:
	if !text.is_valid_hex_number():
		malformed_token.emit()
		return
	var buff:PackedByteArray = text.hex_decode()
	if buff.size() != 2048:
		malformed_token.emit()
		return
	token_entered.emit(buff)

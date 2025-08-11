extends MessageHandler
class_name AvatarChangeHandler

signal avatar_changed(player:int, avatar:int)

func send_message(player:int, avatar:int) -> void:
	var values:Array = [player, avatar]
	var types:Array[int] = []
	
	types.push_back(_get_value_type(null, 0))
	types.push_back(_get_value_type(values[0], 1))
	
	send_message_final(values, types)

func _get_value_type(_previous_value: Variant, idx: int) -> int:
	match idx:
		0:
			return 2
		1:
			return 2
	return -1

func _process_message(values: Array) -> void:
	avatar_changed.emit(values[0], values[1])

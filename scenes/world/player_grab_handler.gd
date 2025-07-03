extends MessageHandler
class_name PlayerGrabHandler

signal player_grabbed(player:int, object:Node)

func send_message(player:int, target:String) -> void:
	var index_path:Array[int] = []
	if target != "":
		index_path = PathHelper.path_to_index_path(target, self)
		if index_path.is_empty():
			push_error("failed to parse path of grabbed node")
	
	
	var values:Array = [player, index_path]
	var types:Array[int] = []
	
	types.push_back(_get_value_type(null, 0))
	types.push_back(_get_value_type(values[0], 1))
	
	send_message_final(values, types)

func _get_value_type(_previous_value: Variant, idx: int) -> int:
	match idx:
		0:
			return 2
		1:
			return 7
	return -1

func _process_message(values: Array) -> void:
	var target:Node = get_tree().root
	if values[1] != []:
		# scene tree could be desynced for us so dont blindly trust the path
		for idx:int in values[1]:
			if target.get_child(idx) == null:
				push_warning("failed to grab node")
				return
			target = target.get_child(idx)
	else:
		target = null
	
	player_grabbed.emit.call_deferred(values[0], target)

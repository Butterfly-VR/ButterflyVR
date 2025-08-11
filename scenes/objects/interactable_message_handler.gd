extends MessageHandler
class_name InteractableHandler

func send_message(target:String, interaction_type:int) -> void:
	var index_path:Array[int] = []
	if target != "":
		index_path = PathHelper.path_to_index_path(target, self)
		if index_path.is_empty():
			push_error("failed to parse path of interacted node")
	
	
	var values:Array = [interaction_type, index_path]
	var types:Array[int] = []
	
	types.push_back(_get_value_type(null, 0))
	types.push_back(_get_value_type(values[0], 1))
	
	send_message_final(values, types)

func _get_value_type(_previous_value: Variant, idx: int) -> int:
	match idx:
		0:
			return 1
		1:
			return 7
	return -1

func _process_message(values: Array) -> void:
	var target:Node = get_tree().root
	if values[1] != []:
		# scene tree could be desynced for us so dont blindly trust the path
		for idx:int in values[1]:
			if target.get_child(idx) == null:
				push_warning("failed to interact with node")
				return
			target = target.get_child(idx)
	else:
		target = null
	if target is Interactable:
		var interactable:Interactable = target
		match values[0]:
			0:
				interactable.interacted_primary.emit()
			1:
				interactable.interacted_secondary.emit()
			2:
				interactable.interacted_tertiary.emit()

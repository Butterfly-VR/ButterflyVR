extends Node
class_name PathHelper

static func path_to_index_path(scene_path:String, caller:Node) -> Array[int]:
	var index_path:Array[int] = []
	var last_find:int = -1
	
	if !scene_path.begins_with("/"):
		push_warning("tried to parse invalid path: " + scene_path)
		return []
	
	# parser needs a trailing slash but no leading slash so we make those changes here
	# also works if the caller already did this for us
	scene_path = scene_path.trim_prefix("/") 
	scene_path = scene_path.trim_suffix("/")
	scene_path += "/"
	
	while true:
		last_find = scene_path.find("/", last_find + 1)
		if last_find == -1:
			break
		var node:Node = caller.get_node("/" + scene_path.substr(0, last_find + 1))
		if node == null:
			push_warning("tried to parse invalid path: " + scene_path)
			return []
		var index:int = node.get_index()
		if index != -1:
			index_path.push_front(index)
	
	if index_path.is_empty():
		push_warning("tried to parse invalid path: " + scene_path)
		return []
	return index_path

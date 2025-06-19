extends Node

const AVATAR_STORE_PATH:String = "user://avatars"
var avatars:Dictionary[int, Avatar] 

class Avatar:
	var pack_path:String
	var avatar_name:String
	var avatar_id:int
	var scene_path:String

func update_avatar_list(clear_existing:bool = false) -> void:
	if clear_existing:
		avatars.clear()
	# load avatars from local files
	if !DirAccess.dir_exists_absolute(AVATAR_STORE_PATH):
		DirAccess.make_dir_recursive_absolute(AVATAR_STORE_PATH)
		const disclaimer_text:String = "WARNING: \nthe current system for loading avatars uses raw .pck files, THESE FILES CAN CONTAIN MALWARE. \nnever put a file here you do not trust, this system will be replaced in the near future with a safer method."
		var disclaimer:FileAccess = FileAccess.open(AVATAR_STORE_PATH + "/DISCLAIMER.txt", FileAccess.WRITE)
		disclaimer.store_string(disclaimer_text)
		disclaimer.close()
		
	for dir_path:String in DirAccess.get_directories_at(AVATAR_STORE_PATH):
		var avatar_info:ConfigFile
		var avatar_pack_location:String
		for file:String in DirAccess.get_files_at(AVATAR_STORE_PATH + "/" + dir_path):
			if file.ends_with(".info"):
				avatar_info = ConfigFile.new()
				avatar_info.load(AVATAR_STORE_PATH + "/" + dir_path + "/" + file)
			if file.ends_with(".pck"):
				avatar_pack_location = AVATAR_STORE_PATH + "/" + dir_path + "/" + file
		if avatar_info != null and avatar_pack_location != null:
			var new_avi:Avatar = Avatar.new()
			var avatar_name:String = avatar_info.get_value("metadata", "name", "UNNAMED")
			if !avatar_info.has_section_key("metadata", "id"):
				continue
			var avatar_id:int = avatar_info.get_value("metadata", "id")
			if !avatar_info.has_section_key("metadata", "scene_path"):
				continue
			var scene_path:String = avatar_info.get_value("metadata", "scene_path")
			if ProjectSettings.load_resource_pack(avatar_pack_location, false):
				new_avi.pack_path = avatar_pack_location
				new_avi.avatar_id = avatar_id
				new_avi.avatar_name = avatar_name
				new_avi.scene_path = scene_path
				avatars[avatar_id] = new_avi

extends VBoxContainer

var cached_avatars:Dictionary[int, AvatarPackLoader.Avatar] = {}
var button:Button = preload("res://scenes/player/ui/avatar_button.tscn").instantiate()

var selected_avatar:int = 0

func update() -> void:
	if is_visible_in_tree():
		AvatarPackLoader.update_avatar_list()
		if cached_avatars != AvatarPackLoader.avatars or cached_avatars == {}:
			for child:Node in get_children():
				child.queue_free()
			await get_tree().process_frame
			cached_avatars = AvatarPackLoader.avatars
			for avatar:AvatarPackLoader.Avatar in cached_avatars.values():
				var avatar_button:Button = button.duplicate()
				avatar_button.text = avatar.avatar_name
				avatar_button.pressed.connect(change_avatar.bind(avatar.avatar_id))
				add_child(avatar_button)
			# handle default avatar
			var default_avatar_button:Button = button.duplicate()
			default_avatar_button.text = "cube fella"
			default_avatar_button.pressed.connect(change_avatar.bind(0))
			add_child(default_avatar_button)
			
func change_avatar(avatar:int) -> void:
	selected_avatar = avatar

func equip_avatar() -> void:
	(NetworkManager as NetNodeManager).change_avatar(selected_avatar)

extends Label

func _ready() -> void:
	@warning_ignore("unsafe_property_access")
	for message:ChatBoxManager.Message in GlobalWorldAccess.current_world.chat_box_manager.messages:
		add_message(message)
	@warning_ignore("unsafe_property_access", "unsafe_method_access")
	GlobalWorldAccess.current_world.chat_box_manager.new_message_sent.connect(add_message)

func add_message(new_message:ChatBoxManager.Message) -> void:
	var player:int = new_message.player
	var player_name:String
	if player == 0:
		player_name = "SYSTEM"
	else:
		player_name = "Player " + str(player)
	var message:String = player_name + ": "
	message += new_message.text
	message += "\n"
	text += message

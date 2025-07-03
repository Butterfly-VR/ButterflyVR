extends Control


func send() -> void:
	var message:String = (get_child(0) as LineEdit).text.strip_escapes()
	if message.is_empty():
		return
	(get_child(0) as LineEdit).clear()
	GlobalWorldAccess.current_world.chat_box_manager.send_message(message)

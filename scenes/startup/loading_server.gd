extends Node
# will later take command arguments / load a config file to control startup
func _ready() -> void:
	print("server starting up")
	if OS.get_name() == "Windows":
		print("sorry, windows servers are currently non functional and cannot be started, a fix for this will happen eventually")
		get_tree().free()
	var bind_ip:String = "127.0.0.1"
	var bind_port:int = 21442
	var key_packed:PackedByteArray = Crypto.new().generate_random_bytes(32)
	var key:Array[int] = []
	var token_number:int = 1
	for byte:int in key_packed:
		key.append(byte)
	for argument:String in OS.get_cmdline_args():
		if argument.begins_with("bind_ip=") and argument.trim_prefix("bind_ip=").is_valid_ip_address():
			bind_ip = argument.trim_prefix("bind_ip=")
		if argument.begins_with("bind_port=") and argument.trim_prefix("bind_port=").is_valid_int():
			var port:int = int(argument.trim_prefix("bind_port="))
			if port >= 1024 and port <= 65535:
				bind_port = port
		if argument.begins_with("token_count=") and argument.trim_prefix("token_count=").is_valid_int():
			token_number = int(argument.trim_prefix("token_count"))
	var bind_addr:String = bind_ip + ":" + str(bind_port)
	print("binding to address: ", bind_addr)
	(NetworkManager as NetNodeManager).start_server(bind_addr, key)
	print("server started. loading world: <debug>")
	for i:int in range(token_number):
		print((NetworkManager as NetNodeManager).get_next_client().hex_encode())
		await get_tree().physics_frame
	get_tree().change_scene_to_file.call_deferred("res://scenes/world/debug_world.tscn")

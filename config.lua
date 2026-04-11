config.pipeline_steps = { "metadata-restorer" };
config.allowed_extensions = { "mp3", "flac", "ogg", "wav" };

config.watch_dir = "./plugins/metadata_restorer/assets/";
config.db_url = "./db/db.sqlite";

-- TODO(pencelheimer): support nested tables in config
config.metadata_restorer_acoustid_api_key = os.getenv("ACOUSTID_API_KEY")
config.metadata_restorer_user_agent = "MusicManager/0.1.0 (pencelheimer@proton.me)"

use super::*;
use crate::alloc::string::ToString;
impl Fat16Impl {
    pub fn new(inner: impl BlockDevice<Block512>) -> Self {
        let mut block = Block::default();
        let block_size = Block512::size();

        inner.read_block(0, &mut block).unwrap();
        let bpb = Fat16Bpb::new(block.as_ref()).unwrap();

        trace!("Loading Fat16 Volume: {:#?}", bpb);

        // HINT: FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let fat_start = bpb.reserved_sector_count() as usize;
        /* FIXME: get the size of root dir from bpb */
        let root_dir_size = (bpb.root_entries_count() as usize * DirEntry::LEN + block_size - 1) / block_size;
        /* FIXME: calculate the first root dir sector */
        let first_root_dir_sector = fat_start + (bpb.fat_count() as usize * bpb.sectors_per_fat() as usize);
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            bpb,
            inner: Box::new(inner),
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }
    //将 cluster: &Cluster 转换为 sector
    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // FIXME: calculate the first sector of the cluster
                // HINT: FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                ((c - 2) as usize * self.bpb.sectors_per_cluster() as usize) + self.first_data_sector
            }
        }
    }

    // FIXME: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //      - read the FAT and get next cluster
    //根据当前 cluster: &Cluster，利用 FAT 表，获取下一个 cluster
    pub fn next_cluster(&self, cluster: Cluster) -> Result<Cluster> {
        let fat_offset = (cluster.0 * 2) as usize;
        let cur_fat_sector = self.fat_start + fat_offset / Block512::size();
        let offset = fat_offset % Block512::size();

        let mut block = Block::default();
        self.inner.read_block(cur_fat_sector, &mut block).unwrap();

        let fat_entry = u16::from_le_bytes(block[offset..=offset + 1].try_into().unwrap_or([0; 2]));
        match fat_entry {
            0xFFF7 => Err(FsError::BadCluster),         // Bad cluster
            0xFFF8..=0xFFFF => Err(FsError::EndOfFile), // There is no next cluster
            f => Ok(Cluster(f as u32)),                     // Seems legit
        }
    }
    //      - traverse the cluster chain and read the data
    //遍历文件夹 dir: &Directory，获取其中文件信息
    pub fn iterate_dir<F>(&self, dir: &Directory, mut func: F) -> Result<()>
    where
        F: FnMut(&DirEntry),
    {
        if let Some(entry) = &dir.entry {
            trace!("Iterating directory: {}", entry.filename());
        }

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        trace!("Directory size: {}", dir_size);

        let mut block = Block::default();
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                self.inner.read_block(sector, &mut block).unwrap();
                for entry in 0..Block512::size() / DirEntry::LEN {
                    let start = entry * DirEntry::LEN;
                    let end = (entry + 1) * DirEntry::LEN;

                    let dir_entry ; // !
                    match DirEntry::parse(&block[start..end]){
                        Ok(entry) => dir_entry = entry,
                        Err(e) => {
                            return Err(e);
                        }
                    }


                    if dir_entry.is_eod() {
                        return Ok(());
                    } else if dir_entry.is_valid() && !dir_entry.is_long_name() {
                        func(&dir_entry);
                    }
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ok(())
    }
    //根据当前文件夹 dir: &Directory 信息，获取名字为 name: &str 的 DirEntry
    pub fn find_directory_entry(
        &self,
        dir: &Directory,
        name: &str,
    ) -> Result<DirEntry> {
        let match_name ;
        match ShortFileName::parse(name){
            Ok(name) => match_name = name,
            Err(e) => return Err(e)
        }

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                match self.find_entry_in_sector(&match_name, sector) {
                    Err(FsError::NotInSector) => continue,
                    x => return x,
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Err(FsError::FileNotFound)
    }
    fn find_entry_in_sector(
        &self,
        match_name: &ShortFileName,
        sector: usize,
    ) -> Result<DirEntry> {
        let mut block = Block::default();
        self.inner.read_block(sector, &mut block).unwrap();

        for entry in 0..Block512::size() / DirEntry::LEN {
            let start = entry * DirEntry::LEN;
            let end = (entry + 1) * DirEntry::LEN;
            let dir_entry =
                DirEntry::parse(&block[start..end]).map_err(|_| FsError::InvalidOperation)?;
            // trace!("Matching {} to {}...", dir_entry.filename(), match_name);
            if dir_entry.is_eod() {
                // Can quit early
                return Err(FsError::FileNotFound);
            } else if dir_entry.filename.matches(match_name) {
                // Found it
                return Ok(dir_entry);
            };
        }
        Err(FsError::NotInSector)
    }
    //      - parse the path
    pub fn resolve_path(&self, root_path: &str) -> Option<Directory> {
        let mut path = root_path.to_owned();
        let mut root = fat16::Fat16Impl::root_dir();

        while let Some(pos) = path.find('/') {
            let dir = path[..pos].to_owned();

            let tmp = self.find_directory_entry(&root, dir.as_str());
            let tmp = if tmp.is_err() {
                warn!("File not found: {}, {:?}", root_path, tmp);
                return None;
            }else{
                tmp.unwrap()
            };
            let tmp = if tmp.is_directory(){
                Ok(Directory::from_entry(tmp))
            }else{
                Err(FsError::NotADirectory)
            };
            if tmp.is_err() {
                warn!("Directory not found: {}, {:?}", root_path, tmp);
                return None;
            }

            root = tmp.unwrap();

            path = path[pos + 1..].to_string();
            trace!("Resolving path: {}", path);

            if path.is_empty() {
                break;
            }
        }

        Some(root)
    }
    //      - open the root directory
    pub fn root_dir() -> Directory {
        Directory::new(Cluster::ROOT_DIR)
    }
    //      - ...
    //      - finally, implement the FileSystem trait for Fat16 with `self.handle`
}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> Result<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read dir and return an iterator for all entries
        let root = match self.handle.resolve_path(path) {
            Some(root) => root,
            None => return Err(FsError::FileNotFound),
        };
        let mut ret: Vec<Metadata> = Vec::new();
        if let Err(err) = self.handle.iterate_dir(&root, |entry| {
            ret.push(entry.try_into().unwrap());
        }) {
            warn!("{:#?}",err);
        }
        Ok(Box::new(ret.into_iter()))
        
    }

    fn open_file(&self, path: &str) -> Result<FileHandle> {
        // FIXME: open file and return a file handle
        info!("{}",path);
        let path = path.to_owned();
        let pos = path.rfind('/');
        let root;
        let filename;
        if pos.is_none() {
            
            root = fat16::Fat16Impl::root_dir();
            filename = path.as_str();
        }else
        {  
            let pos = pos.unwrap();

            trace!("Root: {}, Filename: {}", &path[..=pos], &path[pos + 1..]);

            let rootu = self.handle.resolve_path(&path[..=pos]);
            filename = &path[pos + 1..];
            if rootu.is_none() {
                return Err(FsError::FileNotFound);
            }
            root = rootu.unwrap();
        }

        

        //
        trace!("Try open file: {}", filename);
        let dir_entry = self.handle.find_directory_entry(&root, filename)?;

        if dir_entry.is_directory() {
            return Err(FsError::NotAFile);
        }

        // let file = File {
        //         start_cluster: dir_entry.cluster,
        //         length: dir_entry.size,
        //         mode,
        //         entry: dir_entry,
        //     }
        let file = File::new(self.handle.clone(), dir_entry);

        trace!("Opened file: {:#?}", &file);
        //debug!("{:#?}",self.metadata(&path).unwrap());
        let file_handle = FileHandle::new(self.metadata("APP/").unwrap(), Box::new(file));
        Ok(file_handle)
    }

    fn metadata(&self, root_path: &str) -> Result<Metadata> {
        // FIXME: read metadata of the file / dir
        let mut path = root_path.to_owned();
        let mut root = fat16::Fat16Impl::root_dir();
        let mut ret:Result<Metadata> = Err(FsError::FileNotFound);
        // if path.rfind('/').is_none() {
        //     ret = Ok((&root.entry.unwrap()).try_into().unwrap());
        //     return ret;
        // };
        while let Some(pos) = path.find('/') {
            let dir = path[..pos].to_owned();

            let tmp = self.handle.find_directory_entry(&root, dir.as_str());
            let tmp = if tmp.is_err() {
                warn!("File not found: {}, {:?}", root_path, tmp);
                return Err(FsError::FileNotFound);
            }else{
                tmp.unwrap()
            };
            ret = Ok((&tmp).try_into().unwrap());
            let tmp = if tmp.is_directory(){
                Ok(Directory::from_entry(tmp))
            }else{
                Err(FsError::FileNotFound)
            };
            if tmp.is_err() {
                warn!("Directory not found: {}, {:?}", root_path, tmp);
                return Err(FsError::FileNotFound);
            }

            root = tmp.unwrap();

            path = path[pos + 1..].to_string();
            trace!("Resolving path: {}", path);

            if path.is_empty() {
                break;
            }
        }

        ret
    }

    fn exists(&self, root_path: &str) -> Result<bool> {
        // FIXME: check if the file / dir exists
        let mut path = root_path.to_owned();
        let mut root = fat16::Fat16Impl::root_dir();

        while let Some(pos) = path.find('/') {
            let dir = path[..pos].to_owned();

            let tmp = self.handle.find_directory_entry(&root, dir.as_str());
            let tmp = if tmp.is_err() {
                return Ok(false);
            }else{
                tmp.unwrap()
            };
            let tmp = if tmp.is_directory(){
                Ok(Directory::from_entry(tmp))
            }else{
                Err(FsError::NotADirectory)
            };
            
            path = path[pos + 1..].to_string();
            trace!("Resolving path: {}", path);

            if path.is_empty() {
                break;
            }
            if tmp.is_err() {
                //warn!("Directory not found: {}, {:?}", root_path, tmp);
                return Ok(false);
            }
            root = tmp.unwrap();
        }

        Ok(true)
    }
}

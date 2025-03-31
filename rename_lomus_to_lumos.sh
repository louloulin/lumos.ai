#!/bin/bash

# 创建日志文件
LOG_FILE="rename_lomus_to_lumos.log"
echo "开始重命名过程: $(date)" > $LOG_FILE

# 1. 替换文件内容
echo "正在替换文件内容..." | tee -a $LOG_FILE
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" \) -not -path "*/target/*" -not -path "*/.git/*" | while read file; do
  echo "处理文件: $file" >> $LOG_FILE
  sed -i '' 's/lomusai/lumosai/g' "$file"
  sed -i '' 's/Lomusai/Lumosai/g' "$file"
  sed -i '' 's/lomus/lumos/g' "$file"
  sed -i '' 's/Lomus/Lumos/g' "$file"
done

# 处理package.json文件
find . -type f -name "package.json" -not -path "*/node_modules/*" | while read file; do
  echo "处理文件: $file" >> $LOG_FILE
  sed -i '' 's/"@lomusai/"@lumosai/g' "$file"
  sed -i '' 's/"lomusai"/"lumosai"/g' "$file"
  sed -i '' 's/"Lomusai"/"Lumosai"/g' "$file"
done

# 2. 重命名目录（按深度从高到低排序，避免路径变更问题）
echo "正在重命名目录..." | tee -a $LOG_FILE
find . -depth -type d -name "lomusai_*" -not -path "*/target/*" -not -path "*/.git/*" | sort -r | while read dir; do
  new_dir_name=$(echo "$dir" | sed 's/lomusai_/lumosai_/g')
  if [ "$dir" != "$new_dir_name" ]; then
    mv "$dir" "$new_dir_name"
    echo "重命名目录: $dir -> $new_dir_name" | tee -a $LOG_FILE
  fi
done

# 3. 重命名特定文件
echo "正在重命名特定文件..." | tee -a $LOG_FILE
find . -type f -name "lomusai.md" -not -path "*/target/*" -not -path "*/.git/*" | while read file; do
  new_file_name=$(echo "$file" | sed 's/lomusai/lumosai/g')
  if [ "$file" != "$new_file_name" ]; then
    mv "$file" "$new_file_name"
    echo "重命名文件: $file -> $new_file_name" | tee -a $LOG_FILE
  fi
done

echo "重命名过程完成: $(date)" | tee -a $LOG_FILE
echo "请检查 $LOG_FILE 文件查看详细记录" 
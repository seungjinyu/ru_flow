# train.py
import json

# 테스트용 결과 생성
metrics = {
    "accuracy": 0.85,
    "loss": 0.42
}

# 결과를 metrics.json 파일로 저장
with open("metrics.json", "w") as f:
    json.dump(metrics, f)

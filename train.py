import torch
import torch.nn as nn
import torch.optim as optim
import torchvision.transforms as transforms
import torchvision.datasets as datasets
from torch.utils.data import DataLoader, TensorDataset
import time
import os
from tqdm import tqdm
from torchvision import models

device = torch.device("cuda" if torch.cuda.is_available() else "cpu")


# ✅ ResNet18 간소화 모델 (CIFAR-10용)
def get_resnet18():
    model = models.resnet18(pretrained=False)
    model.conv1 = nn.Conv2d(3, 64, kernel_size=3, stride=1, padding=1, bias=False)
    model.maxpool = nn.Identity()
    model.fc = nn.Linear(512, 10)
    return model


# ✅ 학습 함수
def train(model, train_loader, test_loader, epochs=50):
    model = model.to(device)
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.SGD(model.parameters(), lr=0.01, momentum=0.9)

    for epoch in range(epochs):
        model.train()
        running_loss = 0.0

        progress = tqdm(train_loader, desc=f"Epoch | {epoch+1}/{epochs}",leave=False)

        for images, labels in train_loader:
            images, labels = images.to(device), labels.to(device)
            optimizer.zero_grad()
            outputs =  model(images)
            loss = criterion(model(images), labels)
            loss.backward()
            optimizer.step()

            running_loss += loss.item()
            progress.set_postfix(loss=running_loss / (progress.n +1))
        acc = evaluate(model, test_loader)
        print(f"Epoch {epoch+1}/{epochs} : Test Accuracy = {acc:.2f}%")
    return acc


# ✅ 평가 함수
def evaluate(model, test_loader):
    model.eval()
    correct = 0
    total = 0
    with torch.no_grad():
        for images, labels in test_loader:
            images, labels = images.to(device), labels.to(device)
            outputs = model(images)
            _, predicted = torch.max(outputs.data, 1)
            total += labels.size(0)
            correct += (predicted == labels).sum().item()

    acc = 100 * correct / total
    return acc


# ✅ CIFAR-10 로딩
def get_cifar10_loaders(batch_size=128):
    transform = transforms.Compose([
        transforms.ToTensor(),
    ])
    train_set = datasets.CIFAR10(root='./data/cifar', train=True, download=True, transform=transform)
    test_set = datasets.CIFAR10(root='./data/cifar', train=False, download=True, transform=transform)

    train_loader = DataLoader(train_set, batch_size=batch_size, shuffle=True)
    test_loader = DataLoader(test_set, batch_size=batch_size, shuffle=False)
    return train_loader, test_loader


# ✅ Distilled 데이터 로딩
def get_distilled_loader():
    base_dir = os.path.dirname(os.path.abspath(__file__))
    image_path = os.path.join(base_dir, 'data', 'images_best.pt')
    label_path = os.path.join(base_dir, 'data', 'labels_best.pt')

    image_syn = torch.load(image_path)
    label_syn = torch.load(label_path)

    dataset = TensorDataset(image_syn, label_syn)
    loader = DataLoader(dataset, batch_size=128, shuffle=True)
    return loader


# ✅ 메인 비교 실험
def main():
    print("== Training on Original CIFAR-10 ==")
    model1 = get_resnet18()
    train_loader, test_loader = get_cifar10_loaders()
    start = time.time()
    acc_real = train(model1, train_loader, test_loader, epochs=50)
    print(f"Original Data Accuracy: {acc_real:.2f}%, Time: {time.time() - start:.1f}s")

    print("\n== Training on Distilled Dataset ==")
    model2 = get_resnet18()
    syn_loader = get_distilled_loader()
    start = time.time()
    acc_syn = train(model2, syn_loader, test_loader, epochs=1000)  # 보통 distilled는 더 많이 돌림
    print(f"Distilled Data Accuracy: {acc_syn:.2f}%, Time: {time.time() - start:.1f}s")


if __name__ == "__main__":
    main()

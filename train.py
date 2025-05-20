import argparse
import json
import torch
import torch.nn as nn
import torch.optim as optim
from torchvision import datasets, transforms
from torch.utils.data import DataLoader

# -------------------------------
# Argument parsing
# -------------------------------
parser = argparse.ArgumentParser()
parser.add_argument("--lr", type=float, default=0.01)
parser.add_argument("--batch", type=int, default=64)
args = parser.parse_args()

# -------------------------------
# Dataset and Dataloader
# -------------------------------
transform = transforms.Compose([
    transforms.ToTensor(),
    transforms.Normalize((0.1307,), (0.3081,))
])

train_dataset = datasets.MNIST('.', train=True, download=True, transform=transform)
test_dataset = datasets.MNIST('.', train=False, transform=transform)

train_loader = DataLoader(train_dataset, batch_size=args.batch, shuffle=True)
test_loader = DataLoader(test_dataset, batch_size=1000, shuffle=False)

# -------------------------------
# Simple CNN Model
# -------------------------------
class Net(nn.Module):
    def __init__(self):
        super().__init__()
        self.conv = nn.Sequential(
            nn.Conv2d(1, 16, 3, 1),
            nn.ReLU(),
            nn.MaxPool2d(2),
            nn.Conv2d(16, 32, 3, 1),
            nn.ReLU(),
            nn.MaxPool2d(2),
        )
        self.fc = nn.Sequential(
            nn.Flatten(),
            nn.Linear(5*5*32, 128),
            nn.ReLU(),
            nn.Linear(128, 10),
        )

    def forward(self, x):
        return self.fc(self.conv(x))

model = Net()
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
model.to(device)

# -------------------------------
# Optimizer & Loss
# -------------------------------
criterion = nn.CrossEntropyLoss()
optimizer = optim.SGD(model.parameters(), lr=args.lr)

# -------------------------------
# Training loop (1 epoch)
# -------------------------------
model.train()
for images, labels in train_loader:
    images, labels = images.to(device), labels.to(device)
    optimizer.zero_grad()
    output = model(images)
    loss = criterion(output, labels)
    loss.backward()
    optimizer.step()

# -------------------------------
# Evaluation
# -------------------------------
model.eval()
correct = 0
total = 0
with torch.no_grad():
    for images, labels in test_loader:
        images, labels = images.to(device), labels.to(device)
        outputs = model(images)
        _, predicted = torch.max(outputs, 1)
        correct += (predicted == labels).sum().item()
        total += labels.size(0)

accuracy = correct / total

# -------------------------------
# Save metrics
# -------------------------------
with open("metrics.json", "w") as f:
    json.dump({"accuracy": accuracy}, f)

print(f"✅ Test accuracy: {accuracy:.4f}")

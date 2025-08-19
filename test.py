import smtplib
from email.mime.text import MIMEText

def test_smtp_server(i):
    smtp_host = "0.0.0.0"
    smtp_port = 2525

    sender = "sender@example.com"
    recipient = "receiver@example.com"
    subject = "smpt test"
    body = "test test test imagine i wrote something here."

    msg = MIMEText(body)
    msg["From"] = sender
    msg["To"] = recipient
    msg["Subject"] = subject

    try:
        # print(f"connecting to server at {smtp_host}:{smtp_port}...")
        with smtplib.SMTP(smtp_host, smtp_port, timeout=10) as server:
            server.sendmail(sender, [recipient], msg.as_string())
        print(f"✅ email sent {i}")
    except Exception as e:
        print(f"❌ email failed: {e}")

if __name__ == "__main__":
    i = 1
    while True:
        test_smtp_server(i)
        i += 1

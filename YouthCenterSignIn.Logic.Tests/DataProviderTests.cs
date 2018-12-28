using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class DataProviderTests : TestBase
    {
        private const string NewPin = "321321";

        [TestMethod]
        public void DataProvider_AdminPin_ChangeTest()
        {
            Assert.ThrowsException<MessageException>(() =>
                DataProvider.ChangeAdminPin(NewPin, "1234", "1234"));
            Assert.ThrowsException<MessageException>(() =>
                DataProvider.ChangeAdminPin(Data.DataProvider.DefaultAdminPin, "1234", "2345"));
            Assert.ThrowsException<MessageException>(() =>
                DataProvider.ChangeAdminPin(Data.DataProvider.DefaultAdminPin, "123", "123"));
            Assert.ThrowsException<MessageException>(() =>
                DataProvider.ChangeAdminPin(Data.DataProvider.DefaultAdminPin, NewPin, "4567"));
            Assert.AreEqual(Data.DataProvider.DefaultAdminPin, DataProvider.AdminPin,
                "We tried to change the pin incorrectly, it should not have changed.");

            DataProvider.ChangeAdminPin(Data.DataProvider.DefaultAdminPin, NewPin, NewPin);
            Assert.AreEqual(NewPin, DataProvider.AdminPin, "We changed the pin, it should have updated.");

            DataProvider.ChangeAdminPin(NewPin, Data.DataProvider.DefaultAdminPin, Data.DataProvider.DefaultAdminPin);
        }
        [TestMethod]
        public void DataProvider_AdminPin_AuthenticateTest()
        {
            Assert.IsFalse(DataProvider.AuthenticateAdmin("1243"));
            Assert.IsTrue(DataProvider.AuthenticateAdmin(Data.DataProvider.DefaultAdminPin));
        }
    }
}
